mod auto_recharge;

pub use auto_recharge::AutoRechargeResp;

use crate::error::AppError;
use crate::models::subscription::{
    BalanceResp, BalanceTransactionResp, SubscriptionRecordListItem,
    SubscriptionRecordListResponse, SubscriptionRecordQuery, RECHARGE_GOODS_TITLE, RECHARGE_SKU_ID,
    RECHARGE_SPU_ID,
};
use crate::routes::admin::auth::authorize_admin;
use crate::routes::admin::permissions::{
    SUBSCRIPTION_AUTO_RECHARGE_EXECUTE, SUBSCRIPTION_RECORD_VIEW, SUBSCRIPTION_VIEW,
};
use crate::routes::ApiResponse;
use crate::services::account;
use crate::services::jk_pay;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use sqlx::{MySql, QueryBuilder};
use std::sync::Arc;

/// POST /api/admin/subscription/auto-recharge
pub async fn auto_recharge(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AutoRechargeResp>>, AppError> {
    authorize_admin(&state, &headers, &[SUBSCRIPTION_AUTO_RECHARGE_EXECUTE]).await?;

    tracing::info!("[AutoRecharge] starting auto-recharge job");
    let users = auto_recharge::load_eligible_users(&state).await?;

    tracing::info!("[AutoRecharge] found {} eligible users", users.len());

    let total = users.len();
    let mut summary = auto_recharge::AutoRechargeSummary::new(total);

    for user in users {
        let openid = user.openid.clone();
        let latest_action = auto_recharge::latest_subscription_action(&state, &openid).await?;

        if latest_action != Some(1) {
            tracing::info!(
                "[AutoRecharge] openid={} skipped (action={:?})",
                openid,
                latest_action
            );
            summary.record_skip();
            continue;
        }

        let (_, payment_password) =
            match account::id_card_and_payment_password(&state, &openid).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!("[AutoRecharge] openid={} skipped: {}", openid, e);
                    summary.record_skip();
                    continue;
                }
            };

        if payment_password.is_empty() || user.id_card_number.is_empty() {
            summary.record_skip();
            continue;
        }

        let request_hash = auto_recharge::build_request_hash(&state, &openid, &payment_password);

        if let Some((_balance_after, external_order_no)) =
            auto_recharge::resolve_existing_auto_recharge(&state, &openid, &request_hash).await?
        {
            tracing::info!(
                "[AutoRecharge] openid={} reused existing successful recharge",
                openid
            );
            summary.record_success(openid, external_order_no);
            continue;
        }

        let user_id = account::user_id_by_openid(&state, &openid).await?;
        let order_no = format!(
            "RC{}{:04}",
            chrono::Utc::now().format("%Y%m%d%H%M%S%3f"),
            user_id % 10000
        );
        let amount_yuan = auto_recharge::AUTO_RECHARGE_AMOUNT as f64 / 100.0;
        let recharge_remark = format!("储值充值 {:.2} 元", amount_yuan);
        let spec_info = format!(
            "[{{\"name\":\"充值金额\",\"value\":\"{:.2}元\"}}]",
            amount_yuan
        );

        let goods_image: String =
            sqlx::query_scalar("SELECT primary_image FROM goods WHERE id = ?")
                .bind(RECHARGE_SPU_ID)
                .fetch_optional(&state.db)
                .await?
                .unwrap_or_default();

        let mut tx = state.db.begin().await?;
        let order_insert = sqlx::query(
            "INSERT INTO orders (order_no, user_id, status, total_amount, paid_amount, discount_amount, remark, request_hash) VALUES (?, ?, 0, ?, 0, 0, ?, ?)",
        )
        .bind(&order_no)
        .bind(user_id)
        .bind(auto_recharge::AUTO_RECHARGE_AMOUNT)
        .bind(&recharge_remark)
        .bind(&request_hash)
        .execute(&mut *tx)
        .await?;
        let order_id = order_insert.last_insert_id();

        sqlx::query(
            "INSERT INTO order_items (order_id, order_no, spu_id, sku_id, goods_title, goods_image, spec_info, unit_price, quantity, subtotal) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?)",
        )
        .bind(order_id)
        .bind(&order_no)
        .bind(RECHARGE_SPU_ID)
        .bind(RECHARGE_SKU_ID)
        .bind(RECHARGE_GOODS_TITLE)
        .bind(&goods_image)
        .bind(&spec_info)
        .bind(auto_recharge::AUTO_RECHARGE_AMOUNT)
        .bind(auto_recharge::AUTO_RECHARGE_AMOUNT)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        let mut redis_conn = state.redis_conn().await?;
        let pay_result = jk_pay::jk_pay(
            &mut redis_conn,
            &state.jk_seller_username,
            &state.jk_seller_password,
            &user.id_card_number,
            &payment_password,
            auto_recharge::AUTO_RECHARGE_AMOUNT,
        )
        .await;

        if pay_result.success {
            let mut tx = state.db.begin().await?;

            let updated = sqlx::query(
                "UPDATE orders SET status = 3, paid_amount = ?, external_order_no = ? WHERE id = ? AND request_hash = ? AND status = 0",
            )
            .bind(pay_result.paid_amount)
            .bind(&pay_result.external_order_no)
            .bind(order_id)
            .bind(&request_hash)
            .execute(&mut *tx)
            .await?;

            if updated.rows_affected() == 0 {
                tx.rollback().await?;

                if let Some((_balance_after, external_order_no)) =
                    auto_recharge::resolve_existing_auto_recharge(&state, &openid, &request_hash)
                        .await?
                {
                    summary.record_success(openid.clone(), external_order_no);
                    continue;
                }

                return Err(AppError::InternalError(
                    "Auto recharge order state changed unexpectedly".to_string(),
                ));
            }

            sqlx::query(
                "INSERT INTO balance_accounts (openid, balance) VALUES (?, ?) \
                 ON DUPLICATE KEY UPDATE balance = balance + ?, updated_at = NOW()",
            )
            .bind(&openid)
            .bind(auto_recharge::AUTO_RECHARGE_AMOUNT)
            .bind(auto_recharge::AUTO_RECHARGE_AMOUNT)
            .execute(&mut *tx)
            .await?;

            let balance_after = account::current_balance(&state, &openid).await?
                + auto_recharge::AUTO_RECHARGE_AMOUNT;

            sqlx::query(
                "INSERT INTO balance_transactions \
                 (openid, amount, balance_after, `type`, external_order_no, status, remark, request_hash) \
                 VALUES (?, ?, ?, 1, ?, 1, '自动充值成功', ?)",
            )
            .bind(&openid)
            .bind(auto_recharge::AUTO_RECHARGE_AMOUNT)
            .bind(balance_after)
            .bind(&pay_result.external_order_no)
            .bind(&request_hash)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;

            tracing::info!(
                "[AutoRecharge] success openid={} order_no={} balance_after={}",
                openid,
                order_no,
                balance_after
            );

            summary.record_success(openid, pay_result.external_order_no);
        } else {
            let reason = pay_result
                .fail_reason
                .unwrap_or_else(|| "扣款失败".to_string());

            let balance_now = account::current_balance(&state, &openid).await?;

            let mut tx = state.db.begin().await?;
            sqlx::query(
                "UPDATE orders SET status = 4, remark = ? WHERE id = ? AND request_hash = ?",
            )
            .bind(&reason)
            .bind(order_id)
            .bind(&request_hash)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                "INSERT INTO balance_transactions \
                 (openid, amount, balance_after, `type`, external_order_no, status, remark, request_hash) \
                 VALUES (?, ?, ?, 1, NULL, 0, ?, ?)",
            )
            .bind(&openid)
            .bind(auto_recharge::AUTO_RECHARGE_AMOUNT)
            .bind(balance_now)
            .bind(&reason)
            .bind(&request_hash)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;

            tracing::warn!(
                "[AutoRecharge] failed openid={} order_no={} reason={}",
                openid,
                order_no,
                reason
            );

            summary.record_failure(openid, reason);
        }
    }

    tracing::info!(
        "[AutoRecharge] done total={} success={} fail={} skipped={}",
        total,
        summary.success_count,
        summary.fail_count,
        summary.skipped_count
    );

    Ok(Json(ApiResponse::success(summary.into_resp())))
}

fn append_subscription_record_filters(
    builder: &mut QueryBuilder<MySql>,
    query: &SubscriptionRecordQuery,
) {
    if let Some(ref openid) = query.openid {
        if !openid.is_empty() {
            let openid_like = format!("%{}%", openid);
            builder.push(" AND TRIM(s.openid) LIKE ");
            builder.push_bind(openid_like);
        }
    }

    if let Some(action) = query.action {
        builder.push(" AND s.action = ");
        builder.push_bind(action);
    }

    if let Some(ref start_date) = query.start_date {
        if !start_date.is_empty() {
            builder.push(" AND s.created_at >= ");
            builder.push_bind(start_date.clone());
        }
    }

    if let Some(ref end_date) = query.end_date {
        if !end_date.is_empty() {
            builder.push(" AND s.created_at < DATE_ADD(");
            builder.push_bind(end_date.clone());
            builder.push(", INTERVAL 1 DAY)");
        }
    }
}

/// GET /api/admin/subscription/records
pub async fn list_subscription_records(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<SubscriptionRecordQuery>,
) -> Result<Json<ApiResponse<SubscriptionRecordListResponse>>, AppError> {
    authorize_admin(&state, &headers, &[SUBSCRIPTION_RECORD_VIEW]).await?;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1).min(100);
    let offset = (page - 1) * page_size;

    let mut count_builder = QueryBuilder::<MySql>::new(
        "SELECT COUNT(*) FROM subscription_records s \
         INNER JOIN ( \
             SELECT latest_time.openid_key, MAX(r.id) AS latest_id \
             FROM ( \
                 SELECT TRIM(openid) AS openid_key, MAX(created_at) AS latest_created_at \
                 FROM subscription_records \
                 GROUP BY TRIM(openid) \
             ) latest_time \
             INNER JOIN subscription_records r \
                 ON TRIM(r.openid) = latest_time.openid_key \
                AND r.created_at = latest_time.latest_created_at \
             GROUP BY latest_time.openid_key \
         ) latest ON latest.latest_id = s.id \
         LEFT JOIN wechat_users w ON w.openid = s.openid \
         WHERE 1=1",
    );
    append_subscription_record_filters(&mut count_builder, &query);
    let total: i64 = count_builder
        .build_query_scalar()
        .fetch_one(&state.db)
        .await?;

    let mut list_builder = QueryBuilder::<MySql>::new(
        r#"
        SELECT s.id,
               s.openid,
               COALESCE(w.real_name, '') AS real_name,
               COALESCE(w.phone, '') AS phone,
               s.action,
               CASE s.action WHEN 1 THEN '开启' ELSE '关闭' END AS action_label,
               DATE_FORMAT(s.created_at, '%Y-%m-%d %H:%i:%s') AS created_at
        FROM subscription_records s
        INNER JOIN (
            SELECT latest_time.openid_key, MAX(r.id) AS latest_id
            FROM (
                SELECT TRIM(openid) AS openid_key, MAX(created_at) AS latest_created_at
                FROM subscription_records
                GROUP BY TRIM(openid)
            ) latest_time
            INNER JOIN subscription_records r
                ON TRIM(r.openid) = latest_time.openid_key
               AND r.created_at = latest_time.latest_created_at
            GROUP BY latest_time.openid_key
        ) latest ON latest.latest_id = s.id
        LEFT JOIN wechat_users w ON w.openid = s.openid
        WHERE 1=1
        "#,
    );
    append_subscription_record_filters(&mut list_builder, &query);
    list_builder.push(" ORDER BY s.id DESC LIMIT ");
    list_builder.push_bind(page_size as i64);
    list_builder.push(" OFFSET ");
    list_builder.push_bind(offset as i64);

    let list = list_builder
        .build_query_as::<SubscriptionRecordListItem>()
        .fetch_all(&state.db)
        .await?;

    Ok(Json(ApiResponse::success(SubscriptionRecordListResponse {
        list,
        total,
        page,
        page_size,
    })))
}

/// GET /api/admin/wechat/users/{openid}/balance
pub async fn get_user_balance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(openid): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    authorize_admin(&state, &headers, &[SUBSCRIPTION_VIEW]).await?;

    let balance = account::current_balance(&state, &openid).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "openid": openid,
        "balance": balance
    }))))
}

/// GET /api/admin/wechat/users/{openid}/balance/transactions
pub async fn get_user_transactions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(openid): Path<String>,
) -> Result<Json<ApiResponse<BalanceResp>>, AppError> {
    authorize_admin(&state, &headers, &[SUBSCRIPTION_VIEW]).await?;

    let balance = account::current_balance(&state, &openid).await?;
    let txs = account::recent_balance_transactions(&state, &openid, 200).await?;

    Ok(Json(ApiResponse::success(BalanceResp {
        balance,
        transactions: txs.into_iter().map(BalanceTransactionResp::from).collect(),
    })))
}
