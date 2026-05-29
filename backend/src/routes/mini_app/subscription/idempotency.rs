use super::RechargeResp;
use crate::error::AppError;
use crate::services::account;
use crate::state::AppState;

#[derive(sqlx::FromRow)]
struct ExistingRechargeOrder {
    status: i8,
    total_amount: i64,
    remark: Option<String>,
}

#[derive(sqlx::FromRow)]
struct BalanceAfterRow {
    balance_after: i64,
}

pub(super) async fn resolve_recharge_by_request_id(
    state: &AppState,
    openid: &str,
    request_hash: &str,
) -> Result<Option<RechargeResp>, AppError> {
    let existing = sqlx::query_as::<_, ExistingRechargeOrder>(
        "SELECT status, total_amount, remark \
         FROM orders WHERE request_hash = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(request_hash)
    .fetch_optional(&state.db)
    .await?;

    let Some(order) = existing else {
        return Ok(None);
    };

    let amount_yuan = order.total_amount as f64 / 100.0;

    match order.status {
        0 | 1 => Ok(Some(build_recharge_resp(
            false,
            account::current_balance(state, openid).await?,
            order.total_amount,
            amount_yuan,
            "Recharge is already in progress, please do not submit it again".to_string(),
        ))),
        3 => {
            let balance = match fetch_balance_after_request(state, request_hash, 1).await? {
                Some(balance) => balance,
                None => account::current_balance(state, openid).await?,
            };

            Ok(Some(build_recharge_resp(
                true,
                balance,
                order.total_amount,
                amount_yuan,
                "Recharge successful".to_string(),
            )))
        }
        4 => {
            let balance = match fetch_balance_after_request(state, request_hash, 0).await? {
                Some(balance) => balance,
                None => account::current_balance(state, openid).await?,
            };

            Ok(Some(build_recharge_resp(
                false,
                balance,
                order.total_amount,
                amount_yuan,
                order
                    .remark
                    .unwrap_or_else(|| "Recharge failed".to_string()),
            )))
        }
        _ => Ok(None),
    }
}

async fn fetch_balance_after_request(
    state: &AppState,
    request_hash: &str,
    status: i8,
) -> Result<Option<i64>, AppError> {
    let row = sqlx::query_as::<_, BalanceAfterRow>(
        "SELECT balance_after FROM balance_transactions \
         WHERE request_hash = ? AND status = ? \
         ORDER BY id DESC LIMIT 1",
    )
    .bind(request_hash)
    .bind(status)
    .fetch_optional(&state.db)
    .await?;

    Ok(row.map(|row| row.balance_after))
}

fn build_recharge_resp(
    success: bool,
    balance: i64,
    amount: i64,
    amount_yuan: f64,
    message: String,
) -> RechargeResp {
    RechargeResp {
        success,
        balance,
        amount,
        amount_yuan,
        message,
    }
}
