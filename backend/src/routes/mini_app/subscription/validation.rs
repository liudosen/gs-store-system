use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug)]
pub(super) struct RechargeUser {
    pub(super) id: u64,
    pub(super) id_card_number: String,
}

#[derive(sqlx::FromRow)]
struct RechargeUserRow {
    id: u64,
    id_card_number: String,
}

pub(super) fn validate_recharge_amount(amount: i64) -> Result<(), AppError> {
    if amount < 1 {
        return Err(AppError::BadRequest("充值金额最小 0.01 元".to_string()));
    }

    Ok(())
}

pub(super) fn normalize_request_id(request_id: &str) -> Result<String, AppError> {
    let request_id = request_id.trim();
    if request_id.is_empty() {
        return Err(AppError::BadRequest("requestId is required".to_string()));
    }

    Ok(request_id.to_string())
}

pub(super) async fn load_recharge_user(
    state: &AppState,
    openid: &str,
) -> Result<RechargeUser, AppError> {
    let user = sqlx::query_as::<_, RechargeUserRow>(
        "SELECT id, id_card_number FROM wechat_users WHERE openid = ?",
    )
    .bind(openid)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("用户不存在".to_string()))?;

    if user.id_card_number.is_empty() {
        return Err(AppError::BadRequest("请先绑定身份证号后再充值".to_string()));
    }

    Ok(RechargeUser {
        id: user.id,
        id_card_number: user.id_card_number,
    })
}

#[cfg(test)]
mod tests {
    use super::{normalize_request_id, validate_recharge_amount};
    use crate::error::AppError;

    #[test]
    fn request_id_is_trimmed() {
        assert_eq!(
            normalize_request_id("  req-123  ").expect("request id"),
            "req-123"
        );
    }

    #[test]
    fn empty_request_id_is_rejected() {
        let err = normalize_request_id("   ").expect_err("request id");
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn recharge_amount_must_be_positive() {
        assert!(validate_recharge_amount(1).is_ok());
        let err = validate_recharge_amount(0).expect_err("amount");
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}
