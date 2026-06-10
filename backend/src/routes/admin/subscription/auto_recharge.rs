use crate::error::AppError;
use crate::services::{account, secret};
use crate::state::AppState;
use serde::Serialize;

pub const AUTO_RECHARGE_AMOUNT: i64 = 200_000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoRechargeResp {
    pub total: usize,
    pub success_count: usize,
    pub fail_count: usize,
    pub skipped_count: usize,
    pub results: Vec<AutoRechargeUserResult>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoRechargeUserResult {
    pub openid: String,
    pub success: bool,
    pub fail_reason: Option<String>,
    pub external_order_no: Option<String>,
}

#[derive(Default)]
pub(super) struct AutoRechargeSummary {
    pub(super) total: usize,
    pub(super) success_count: usize,
    pub(super) fail_count: usize,
    pub(super) skipped_count: usize,
    results: Vec<AutoRechargeUserResult>,
}

impl AutoRechargeSummary {
    pub(super) fn new(total: usize) -> Self {
        Self {
            total,
            ..Self::default()
        }
    }

    pub(super) fn record_success(&mut self, openid: String, external_order_no: Option<String>) {
        self.success_count += 1;
        self.results.push(AutoRechargeUserResult {
            openid,
            success: true,
            fail_reason: None,
            external_order_no,
        });
    }

    pub(super) fn record_failure(&mut self, openid: String, reason: String) {
        self.fail_count += 1;
        self.results.push(AutoRechargeUserResult {
            openid,
            success: false,
            fail_reason: Some(reason),
            external_order_no: None,
        });
    }

    pub(super) fn record_skip(&mut self) {
        self.skipped_count += 1;
    }

    pub(super) fn into_resp(self) -> AutoRechargeResp {
        AutoRechargeResp {
            total: self.total,
            success_count: self.success_count,
            fail_count: self.fail_count,
            skipped_count: self.skipped_count,
            results: self.results,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct EligibleUser {
    pub openid: String,
    pub id_card_number: String,
}

#[derive(sqlx::FromRow)]
struct EligibleUserRow {
    openid: String,
    id_card_number: String,
}

#[derive(sqlx::FromRow)]
struct ExistingRechargeOrder {
    id: u64,
    status: i8,
    external_order_no: Option<String>,
}

#[derive(sqlx::FromRow)]
struct TxRow {
    balance_after: i64,
}

pub(super) async fn load_eligible_users(state: &AppState) -> Result<Vec<EligibleUser>, AppError> {
    let users = sqlx::query_as::<_, EligibleUserRow>(
        "SELECT openid, id_card_number FROM wechat_users WHERE id_card_number != '' AND payment_password != ''",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(users
        .into_iter()
        .map(|user| EligibleUser {
            openid: user.openid,
            id_card_number: account::normalize_identity_no(&user.id_card_number),
        })
        .collect())
}

pub(super) async fn latest_subscription_action(
    state: &AppState,
    openid: &str,
) -> Result<Option<i8>, AppError> {
    let latest_action: Option<i8> = sqlx::query_scalar(
        "SELECT action FROM subscription_records WHERE openid = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(openid)
    .fetch_optional(&state.db)
    .await?;

    Ok(latest_action)
}

pub(super) fn build_request_hash(
    state: &AppState,
    identity_no: &str,
    payment_password: &str,
) -> String {
    secret::recharge_request_hash(
        &state.jwt_secret,
        identity_no,
        AUTO_RECHARGE_AMOUNT,
        payment_password,
    )
}

pub(super) async fn resolve_existing_auto_recharge(
    state: &AppState,
    identity_no: &str,
    request_hash: &str,
) -> Result<Option<(i64, Option<String>)>, AppError> {
    let existing = sqlx::query_as::<_, ExistingRechargeOrder>(
        "SELECT id, status, external_order_no FROM orders WHERE request_hash = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(request_hash)
    .fetch_optional(&state.db)
    .await?;

    let Some(order) = existing else {
        return Ok(None);
    };

    match order.status {
        0 | 1 => Err(AppError::BadRequest(
            "Recharge is already in progress, please do not submit it again".to_string(),
        )),
        3 => {
            let tx = sqlx::query_as::<_, TxRow>(
                "SELECT balance_after FROM identity_balance_transactions \
                 WHERE request_hash = ? AND status = 1 \
                 ORDER BY id DESC LIMIT 1",
            )
            .bind(request_hash)
            .fetch_optional(&state.db)
            .await?;

            let balance = match tx {
                Some(row) => row.balance_after,
                None => account::current_balance_by_identity_no(state, identity_no).await?,
            };

            Ok(Some((balance, order.external_order_no)))
        }
        4 => {
            sqlx::query("UPDATE orders SET status = 4 WHERE id = ? AND status = 4")
                .bind(order.id)
                .execute(&state.db)
                .await?;
            Ok(None)
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::AutoRechargeSummary;

    #[test]
    fn summary_tracks_outcomes() {
        let mut summary = AutoRechargeSummary::new(3);
        summary.record_success("openid-1".to_string(), Some("order-1".to_string()));
        summary.record_failure("openid-2".to_string(), "insufficient balance".to_string());
        summary.record_skip();

        let resp = summary.into_resp();
        assert_eq!(resp.total, 3);
        assert_eq!(resp.success_count, 1);
        assert_eq!(resp.fail_count, 1);
        assert_eq!(resp.skipped_count, 1);
        assert_eq!(resp.results.len(), 2);
    }
}
