use crate::error::AppError;
use crate::models::subscription::SubscriptionStatusResp;
use crate::services::account;
use crate::state::AppState;

const SUBSCRIPTION_NOT_READY_MESSAGE: &str = "请先完成一次正常购买后再开启订阅";

#[derive(sqlx::FromRow)]
struct SubscriptionRow {
    action: i8,
    created_at: chrono::NaiveDateTime,
}

pub(super) fn validate_subscription_action(action: i8) -> Result<(), AppError> {
    if action != 0 && action != 1 {
        return Err(AppError::BadRequest(
            "订阅状态参数必须是 0 或 1".to_string(),
        ));
    }

    Ok(())
}

pub(super) async fn ensure_subscription_activation_ready(
    state: &AppState,
    openid: &str,
) -> Result<(), AppError> {
    match account::id_card_and_payment_password(state, openid).await {
        Err(_) => Err(AppError::BadRequest(
            SUBSCRIPTION_NOT_READY_MESSAGE.to_string(),
        )),
        Ok((id_card_number, payment_password))
            if id_card_number.is_empty() || payment_password.is_empty() =>
        {
            Err(AppError::BadRequest(
                SUBSCRIPTION_NOT_READY_MESSAGE.to_string(),
            ))
        }
        Ok(_) => Ok(()),
    }
}

pub(super) async fn can_activate_subscription(state: &AppState, openid: &str) -> bool {
    match account::id_card_and_payment_password(state, openid).await {
        Ok((id_card_number, payment_password)) => {
            !id_card_number.is_empty() && !payment_password.is_empty()
        }
        Err(_) => false,
    }
}

pub(super) async fn load_subscription_status(
    state: &AppState,
    openid: &str,
) -> Result<SubscriptionStatusResp, AppError> {
    let row = sqlx::query_as::<_, SubscriptionRow>(
        "SELECT action, created_at FROM subscription_records \
         WHERE openid = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(openid)
    .fetch_optional(&state.db)
    .await?;

    Ok(match row {
        Some(r) => SubscriptionStatusResp {
            action: Some(r.action),
            created_at: Some(r.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
        },
        None => SubscriptionStatusResp {
            action: None,
            created_at: None,
        },
    })
}

pub(super) fn build_subscription_status(action: i8) -> SubscriptionStatusResp {
    SubscriptionStatusResp {
        action: Some(action),
        created_at: Some(current_timestamp_string()),
    }
}

pub(super) fn subscription_not_ready_message() -> &'static str {
    SUBSCRIPTION_NOT_READY_MESSAGE
}

fn current_timestamp_string() -> String {
    chrono::Utc::now()
        .naive_utc()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::validate_subscription_action;
    use crate::error::AppError;

    #[test]
    fn subscription_action_must_be_zero_or_one() {
        assert!(validate_subscription_action(0).is_ok());
        assert!(validate_subscription_action(1).is_ok());

        let err = validate_subscription_action(2).expect_err("invalid action");
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}
