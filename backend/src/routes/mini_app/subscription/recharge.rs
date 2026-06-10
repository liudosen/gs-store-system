use super::balance;
use super::validation::RechargeUser;
use super::RechargeResp;
use crate::error::AppError;
use crate::models::subscription::{RECHARGE_GOODS_TITLE, RECHARGE_SKU_ID, RECHARGE_SPU_ID};
use crate::services::jk_pay;
use crate::state::AppState;
use std::time::Instant;

pub(super) struct RechargeOrderContext {
    pub(super) order_id: u64,
    pub(super) order_no: String,
    pub(super) amount_yuan: f64,
}

pub(super) async fn process_recharge(
    state: &AppState,
    openid: &str,
    user: &RechargeUser,
    amount: i64,
    payment_password: &str,
    request_hash: &str,
    started: Instant,
) -> Result<RechargeResp, AppError> {
    let identity_no = &user.id_card_number;
    let order = create_recharge_order(state, user.id, amount, request_hash).await?;

    let mut redis_conn = state.redis_conn().await?;
    let pay_result = jk_pay::jk_pay_exact_amount(
        &mut redis_conn,
        &state.jk_seller_username,
        &state.jk_seller_password,
        &user.id_card_number,
        payment_password,
        amount,
    )
    .await;

    if pay_result.success {
        finalize_recharge_success(
            state,
            openid,
            identity_no,
            request_hash,
            &order,
            amount,
            payment_password,
            pay_result.paid_amount,
            pay_result.external_order_no.as_deref().unwrap_or(""),
            started,
        )
        .await
    } else {
        let reason = pay_result
            .fail_reason
            .unwrap_or_else(|| "充值失败".to_string());

        finalize_recharge_failure(
            state,
            openid,
            identity_no,
            request_hash,
            &order,
            amount,
            reason,
            started,
        )
        .await
    }
}

async fn create_recharge_order(
    state: &AppState,
    user_id: u64,
    amount: i64,
    request_hash: &str,
) -> Result<RechargeOrderContext, AppError> {
    let order_no = build_recharge_order_no(user_id);
    let amount_yuan = amount as f64 / 100.0;
    let recharge_remark = format!("储值充值{:.2}元", amount_yuan);
    let spec_info = format!(
        "[{{\"name\":\"充值金额\",\"value\":\"{:.2}元\"}}]",
        amount_yuan
    );

    let goods_image: String = sqlx::query_scalar("SELECT primary_image FROM goods WHERE id = ?")
        .bind(RECHARGE_SPU_ID)
        .fetch_optional(&state.db)
        .await?
        .unwrap_or_default();

    let mut tx = state.db.begin().await?;
    let order_insert = sqlx::query(
        "INSERT INTO orders (order_no, user_id, status, total_amount, paid_amount, \
         discount_amount, remark, request_hash) VALUES (?, ?, 0, ?, 0, 0, ?, ?)",
    )
    .bind(&order_no)
    .bind(user_id)
    .bind(amount)
    .bind(&recharge_remark)
    .bind(request_hash)
    .execute(&mut *tx)
    .await?;

    let order_id: u64 = order_insert.last_insert_id();

    sqlx::query(
        "INSERT INTO order_items (order_id, order_no, spu_id, sku_id, goods_title, \
         goods_image, spec_info, unit_price, quantity, subtotal) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?)",
    )
    .bind(order_id)
    .bind(&order_no)
    .bind(RECHARGE_SPU_ID)
    .bind(RECHARGE_SKU_ID)
    .bind(RECHARGE_GOODS_TITLE)
    .bind(&goods_image)
    .bind(&spec_info)
    .bind(amount)
    .bind(amount)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(
        "[Recharge] order created order_no={} amount={} request_hash={}",
        order_no,
        amount,
        request_hash
    );

    Ok(RechargeOrderContext {
        order_id,
        order_no,
        amount_yuan,
    })
}

async fn finalize_recharge_success(
    state: &AppState,
    openid: &str,
    identity_no: &str,
    request_hash: &str,
    order: &RechargeOrderContext,
    amount: i64,
    payment_password: &str,
    paid_amount: i64,
    external_order_no: &str,
    started: Instant,
) -> Result<RechargeResp, AppError> {
    let mut tx = state.db.begin().await?;

    balance::ensure_balance_account(&mut tx, identity_no).await?;
    let balance_before = balance::load_balance_for_update(&mut tx, identity_no).await?;
    let credited_amount = paid_amount.max(0);
    let balance_after = balance_before + credited_amount;
    let success_remark = if credited_amount == amount {
        "recharge success".to_string()
    } else {
        format!(
            "recharge success with actual deduction: requested={:.2}, paid={:.2}, credited={:.2}",
            amount as f64 / 100.0,
            paid_amount as f64 / 100.0,
            credited_amount as f64 / 100.0
        )
    };

    let updated = sqlx::query(
        "UPDATE orders SET status = 3, paid_amount = ?, external_order_no = ?, remark = ? \
         WHERE id = ? AND request_hash = ? AND status = 0",
    )
    .bind(paid_amount)
    .bind(external_order_no)
    .bind(&success_remark)
    .bind(order.order_id)
    .bind(request_hash)
    .execute(&mut *tx)
    .await?;

    if updated.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(AppError::InternalError(
            "Recharge order state changed unexpectedly".to_string(),
        ));
    }

    sqlx::query(
        "UPDATE identity_balance_accounts SET balance = ?, updated_at = NOW() WHERE identity_no = ?",
    )
        .bind(balance_after)
        .bind(identity_no)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO identity_balance_transactions \
         (identity_no, source_openid, amount, balance_after, `type`, external_order_no, status, remark, request_hash) \
         VALUES (?, ?, ?, ?, 1, ?, 1, '主动充值成功', ?)",
    )
    .bind(identity_no)
    .bind(openid)
    .bind(credited_amount)
    .bind(balance_after)
    .bind(external_order_no)
    .bind(request_hash)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE wechat_users SET payment_password = ? WHERE openid = ?")
        .bind(payment_password)
        .bind(openid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    tracing::info!(
        "[Recharge] success openid={} identity_no={} order_no={} balance_after={} elapsed_ms={}",
        openid,
        identity_no,
        order.order_no,
        balance_after,
        started.elapsed().as_millis()
    );

    Ok(RechargeResp {
        success: true,
        balance: balance_after,
        amount: credited_amount,
        amount_yuan: credited_amount as f64 / 100.0,
        message: if credited_amount == amount {
            "充值成功".to_string()
        } else {
            "充值成功，已按健康卡实际扣款金额入账".to_string()
        },
    })
}

async fn finalize_recharge_failure(
    state: &AppState,
    openid: &str,
    identity_no: &str,
    request_hash: &str,
    order: &RechargeOrderContext,
    amount: i64,
    reason: String,
    started: Instant,
) -> Result<RechargeResp, AppError> {
    let mut tx = state.db.begin().await?;

    balance::ensure_balance_account(&mut tx, identity_no).await?;
    let balance_now = balance::load_balance_for_update(&mut tx, identity_no).await?;

    sqlx::query("UPDATE orders SET status = 4, remark = ? WHERE id = ? AND request_hash = ?")
        .bind(&reason)
        .bind(order.order_id)
        .bind(request_hash)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO identity_balance_transactions \
         (identity_no, source_openid, amount, balance_after, `type`, external_order_no, status, remark, request_hash) \
         VALUES (?, ?, ?, ?, 1, NULL, 0, ?, ?)",
    )
    .bind(identity_no)
    .bind(openid)
    .bind(amount)
    .bind(balance_now)
    .bind(&reason)
    .bind(request_hash)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::warn!(
        "[Recharge] failed openid={} identity_no={} order_no={} reason={} elapsed_ms={}",
        openid,
        identity_no,
        order.order_no,
        reason,
        started.elapsed().as_millis()
    );

    Ok(RechargeResp {
        success: false,
        balance: balance_now,
        amount,
        amount_yuan: order.amount_yuan,
        message: reason,
    })
}

fn build_recharge_order_no(user_id: u64) -> String {
    format!(
        "RC{}{:04}",
        chrono::Utc::now().format("%Y%m%d%H%M%S%3f"),
        user_id % 10000
    )
}
