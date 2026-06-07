use crate::error::AppError;
use crate::models::order::{BalancePayRequest, BalancePayResp, PayOrderRequest};
use crate::routes::mini_app::order::{guards, mapping, queries, PayOrderResp};
use crate::routes::ApiResponse;
use crate::services::jk_pay;
use crate::state::AppState;
use axum::Json;

pub(crate) fn calc_balance_payment_amount(
    total_amount_fen: i64,
    discount_amount_fen: i64,
) -> i64 {
    let payable_amount_fen = total_amount_fen
        .max(0)
        .saturating_sub(discount_amount_fen.max(0))
        .max(0);

    ((payable_amount_fen as f64) / 0.95).round() as i64
}

pub(super) fn build_balance_trade_no(order_id: u64) -> String {
    format!(
        "BAL{}{}",
        chrono::Utc::now().format("%Y%m%d%H%M%S%3f"),
        order_id
    )
}

pub(super) async fn pay_order_with_balance_impl(
    state: &AppState,
    openid: &str,
    id: u64,
    _body: BalancePayRequest,
) -> Result<Json<ApiResponse<BalancePayResp>>, AppError> {
    let user_id = queries::get_user_id_by_openid(state, openid).await?;
    let lock_key = format!("balance_pay:{openid}");
    let mut tx = state.db.begin().await?;

    let lock_acquired: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, 5)")
        .bind(&lock_key)
        .fetch_one(&mut *tx)
        .await?;
    if lock_acquired != Some(1) {
        tx.rollback().await?;
        return Err(AppError::BadRequest(
            "payment lock busy, please retry".to_string(),
        ));
    }

    let result = async {
        let order = queries::fetch_owned_order(state, id, user_id).await?;
        let current_balance = queries::fetch_current_balance_on(&mut *tx, openid).await?;

        if order.status != 0 {
            return Ok(Json(ApiResponse::success(mapping::build_balance_pay_resp(
                false,
                0,
                current_balance,
                Some(order.status as i64),
                "only pending orders can be paid",
            ))));
        }

        let balance_paid_amount =
            calc_balance_payment_amount(order.total_amount, order.discount_amount);
        if current_balance < balance_paid_amount {
            return Ok(Json(ApiResponse::success(mapping::build_balance_pay_resp(
                false,
                0,
                current_balance,
                Some(order.status as i64),
                "insufficient balance",
            ))));
        }

        let balance_after = current_balance - balance_paid_amount;
        let balance_trade_no = build_balance_trade_no(order.id);

        let updated = sqlx::query(
            "UPDATE orders SET status = 1, paid_amount = ?, external_order_no = ? WHERE id = ? AND status = 0",
        )
        .bind(balance_paid_amount)
        .bind(&balance_trade_no)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        if updated.rows_affected() != 1 {
            return Err(AppError::BadRequest(
                "order status changed, please retry".to_string(),
            ));
        }

        sqlx::query(
            "INSERT INTO balance_accounts (openid, balance) VALUES (?, ?) ON DUPLICATE KEY UPDATE balance = VALUES(balance), updated_at = NOW()",
        )
        .bind(openid)
        .bind(balance_after)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO balance_transactions (openid, amount, balance_after, `type`, external_order_no, status, remark) VALUES (?, ?, ?, 2, ?, 1, 'order balance payment')",
        )
        .bind(openid)
        .bind(balance_paid_amount)
        .bind(balance_after)
        .bind(&balance_trade_no)
        .execute(&mut *tx)
        .await?;

        Ok(Json(ApiResponse::success(mapping::build_balance_pay_resp(
            true,
            balance_paid_amount,
            balance_after,
            Some(1),
            "payment successful",
        ))))
    }
    .await;

    let _ = sqlx::query_scalar::<_, Option<i32>>("SELECT RELEASE_LOCK(?)")
        .bind(&lock_key)
        .fetch_one(&mut *tx)
        .await;

    match result {
        Ok(resp) => {
            tx.commit().await?;
            Ok(resp)
        }
        Err(err) => {
            tx.rollback().await?;
            Err(err)
        }
    }
}

pub(super) async fn pay_order_impl(
    state: &AppState,
    openid: &str,
    id: u64,
    body: PayOrderRequest,
) -> Result<Json<ApiResponse<PayOrderResp>>, AppError> {
    let payment_password = body.payment_password;
    guards::with_payment_submit_guard(state, openid, async move {
        let user_id = queries::get_user_id_by_openid(state, openid).await?;
        let order = queries::fetch_owned_order(state, id, user_id).await?;

        if order.status != 0 {
            return Err(AppError::BadRequest(
                "only pending orders can be paid".to_string(),
            ));
        }

        let id_card_number = queries::fetch_user_id_card_number(state, openid).await?;
        if id_card_number.is_empty() {
            let fail_msg = "health card id not found, complete profile before payment";
            sqlx::query("UPDATE orders SET remark = ? WHERE id = ?")
                .bind(fail_msg)
                .bind(id)
                .execute(&state.db)
                .await?;
            return Ok(Json(ApiResponse::success(mapping::build_pay_order_resp(
                false, 0, None, fail_msg,
            ))));
        }

        let mut redis_conn = state.redis_conn().await?;
        let result = jk_pay::jk_pay(
            &mut redis_conn,
            &state.jk_seller_username,
            &state.jk_seller_password,
            &id_card_number,
            &payment_password,
            order.total_amount,
        )
        .await;

        if result.success {
            sqlx::query(
                "UPDATE orders SET status = 1, paid_amount = ?, external_order_no = ? WHERE id = ?",
            )
            .bind(result.paid_amount)
            .bind(&result.external_order_no)
            .bind(id)
            .execute(&state.db)
            .await?;

            sqlx::query("UPDATE wechat_users SET payment_password = ? WHERE openid = ?")
                .bind(&payment_password)
                .bind(openid)
                .execute(&state.db)
                .await?;

            Ok(Json(ApiResponse::success(mapping::build_pay_order_resp(
                true,
                result.paid_amount,
                result.order_status,
                "payment successful",
            ))))
        } else {
            let fail_msg = result
                .fail_reason
                .unwrap_or_else(|| "payment failed".to_string());
            sqlx::query("UPDATE orders SET remark = ? WHERE id = ?")
                .bind(&fail_msg)
                .bind(id)
                .execute(&state.db)
                .await?;

            Ok(Json(ApiResponse::success(mapping::build_pay_order_resp(
                false,
                0,
                result.order_status,
                fail_msg,
            ))))
        }
    })
    .await
}
