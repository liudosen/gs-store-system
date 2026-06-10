use crate::error::AppError;
use crate::state::AppState;

pub fn normalize_identity_no(identity_no: &str) -> String {
    identity_no.trim().to_uppercase()
}

pub async fn user_id_by_openid(state: &AppState, openid: &str) -> Result<u64, AppError> {
    let user_id: u64 = sqlx::query_scalar("SELECT id FROM wechat_users WHERE openid = ?")
        .bind(openid)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;
    Ok(user_id)
}

pub async fn id_card_and_payment_password(
    state: &AppState,
    openid: &str,
) -> Result<(String, String), AppError> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id_card_number: String,
        payment_password: String,
    }

    let row = sqlx::query_as::<_, Row>(
        "SELECT id_card_number, payment_password FROM wechat_users WHERE openid = ?",
    )
    .bind(openid)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok((
        normalize_identity_no(&row.id_card_number),
        row.payment_password,
    ))
}

pub async fn real_name_and_id_card_number(
    state: &AppState,
    openid: &str,
) -> Result<(String, String), AppError> {
    #[derive(sqlx::FromRow)]
    struct Row {
        real_name: String,
        id_card_number: String,
    }

    let row = sqlx::query_as::<_, Row>(
        "SELECT real_name, id_card_number FROM wechat_users WHERE openid = ?",
    )
    .bind(openid)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok((row.real_name, normalize_identity_no(&row.id_card_number)))
}

pub async fn identity_no_by_openid(state: &AppState, openid: &str) -> Result<String, AppError> {
    let identity_no: Option<String> =
        sqlx::query_scalar("SELECT id_card_number FROM wechat_users WHERE openid = ?")
            .bind(openid)
            .fetch_optional(&state.db)
            .await?;

    let identity_no = identity_no
        .map(|value| normalize_identity_no(&value))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::BadRequest("请先完成认证号认证后再使用储值余额".to_string()))?;

    Ok(identity_no)
}

pub async fn current_balance(state: &AppState, openid: &str) -> Result<i64, AppError> {
    let identity_no = identity_no_by_openid(state, openid).await?;
    current_balance_by_identity_no(state, &identity_no).await
}

pub async fn current_balance_by_identity_no(
    state: &AppState,
    identity_no: &str,
) -> Result<i64, AppError> {
    let identity_no = normalize_identity_no(identity_no);
    if identity_no.is_empty() {
        return Err(AppError::BadRequest("认证号不能为空".to_string()));
    }

    let balance: i64 = sqlx::query_scalar(
        "SELECT COALESCE( \
             (SELECT balance FROM identity_balance_accounts WHERE identity_no = ?), \
             (SELECT balance_after FROM identity_balance_transactions WHERE identity_no = ? ORDER BY id DESC LIMIT 1), \
             0 \
         )",
    )
    .bind(&identity_no)
    .bind(&identity_no)
    .fetch_one(&state.db)
    .await?;

    Ok(balance)
}

#[allow(dead_code)]
pub async fn recent_balance_transactions(
    state: &AppState,
    openid: &str,
    limit: i64,
) -> Result<Vec<crate::models::subscription::BalanceTransaction>, AppError> {
    let identity_no = identity_no_by_openid(state, openid).await?;
    recent_balance_transactions_by_identity_no(state, &identity_no, limit).await
}

pub async fn recent_balance_transactions_by_identity_no(
    state: &AppState,
    identity_no: &str,
    limit: i64,
) -> Result<Vec<crate::models::subscription::BalanceTransaction>, AppError> {
    let identity_no = normalize_identity_no(identity_no);
    if identity_no.is_empty() {
        return Err(AppError::BadRequest("认证号不能为空".to_string()));
    }

    let txs = sqlx::query_as::<_, crate::models::subscription::BalanceTransaction>(
        "SELECT id, identity_no AS openid, amount, balance_after, `type`, external_order_no, \
         status, remark, created_at \
         FROM identity_balance_transactions \
         WHERE identity_no = ? ORDER BY id DESC LIMIT ?",
    )
    .bind(&identity_no)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    Ok(txs)
}
