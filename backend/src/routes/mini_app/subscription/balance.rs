use crate::error::AppError;
use crate::models::subscription::{BalanceResp, BalanceTransactionResp};
use crate::services::account;
use crate::state::AppState;

pub(super) async fn load_balance_response(
    state: &AppState,
    openid: &str,
) -> Result<BalanceResp, AppError> {
    let identity_no = account::identity_no_by_openid(state, openid).await?;
    let balance = account::current_balance_by_identity_no(state, &identity_no).await?;
    let txs = account::recent_balance_transactions_by_identity_no(state, &identity_no, 50).await?;

    Ok(BalanceResp {
        balance,
        transactions: txs.into_iter().map(BalanceTransactionResp::from).collect(),
    })
}

pub(super) async fn ensure_balance_account(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    identity_no: &str,
) -> Result<(), AppError> {
    let identity_no = account::normalize_identity_no(identity_no);
    if identity_no.is_empty() {
        return Err(AppError::BadRequest("认证号不能为空".to_string()));
    }

    sqlx::query(
        "INSERT IGNORE INTO identity_balance_accounts (identity_no, balance) VALUES (?, 0)",
    )
    .bind(identity_no)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(super) async fn load_balance_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    identity_no: &str,
) -> Result<i64, AppError> {
    let identity_no = account::normalize_identity_no(identity_no);
    if identity_no.is_empty() {
        return Err(AppError::BadRequest("认证号不能为空".to_string()));
    }

    let balance: i64 = sqlx::query_scalar(
        "SELECT balance FROM identity_balance_accounts WHERE identity_no = ? FOR UPDATE",
    )
    .bind(identity_no)
    .fetch_one(&mut **tx)
    .await?;

    Ok(balance)
}
