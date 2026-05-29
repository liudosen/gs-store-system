use crate::error::AppError;
use crate::models::subscription::{BalanceResp, BalanceTransactionResp};
use crate::services::account;
use crate::state::AppState;

pub(super) async fn load_balance_response(
    state: &AppState,
    openid: &str,
) -> Result<BalanceResp, AppError> {
    let balance = account::current_balance(state, openid).await?;
    let txs = account::recent_balance_transactions(state, openid, 50).await?;

    Ok(BalanceResp {
        balance,
        transactions: txs.into_iter().map(BalanceTransactionResp::from).collect(),
    })
}

pub(super) async fn ensure_balance_account(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    openid: &str,
) -> Result<(), AppError> {
    sqlx::query("INSERT IGNORE INTO balance_accounts (openid, balance) VALUES (?, 0)")
        .bind(openid)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(super) async fn load_balance_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    openid: &str,
) -> Result<i64, AppError> {
    let balance: i64 =
        sqlx::query_scalar("SELECT balance FROM balance_accounts WHERE openid = ? FOR UPDATE")
            .bind(openid)
            .fetch_one(&mut **tx)
            .await?;

    Ok(balance)
}
