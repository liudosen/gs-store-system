mod balance;
mod idempotency;
mod recharge;
mod status;
mod validation;

use crate::error::AppError;
use crate::models::subscription::{BalanceResp, SetSubscriptionRequest, SubscriptionStatusResp};
use crate::routes::mini_app::auth::validate_wechat_user;
use crate::routes::ApiResponse;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

/// POST /api/mini/subscription - 开启/关闭订阅
pub async fn set_subscription(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SetSubscriptionRequest>,
) -> Result<Json<ApiResponse<SubscriptionStatusResp>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;
    status::validate_subscription_action(body.action)?;

    if body.action == 1 {
        status::ensure_subscription_activation_ready(&state, &openid).await?;
    }

    sqlx::query("INSERT INTO subscription_records (openid, action) VALUES (?, ?)")
        .bind(&openid)
        .bind(body.action)
        .execute(&state.db)
        .await?;

    tracing::info!("[Subscription] openid={} action={}", openid, body.action);

    Ok(Json(ApiResponse::success(
        status::build_subscription_status(body.action),
    )))
}

/// GET /api/mini/subscription/ability - check whether user can subscribe
pub async fn check_subscription_ability(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;
    let able = status::can_activate_subscription(&state, &openid).await;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "able": able,
        "reason": if able { "" } else { status::subscription_not_ready_message() }
    }))))
}

pub async fn get_subscription(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<SubscriptionStatusResp>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;
    let resp = status::load_subscription_status(&state, &openid).await?;

    Ok(Json(ApiResponse::success(resp)))
}

/// POST /api/mini/recharge request body
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RechargeRequest {
    /// amount in fen
    pub amount: i64,
    /// payment password
    pub payment_password: String,
    /// client-generated idempotency key
    pub request_id: String,
}

/// POST /api/mini/recharge response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RechargeResp {
    pub success: bool,
    pub balance: i64,
    pub amount: i64,
    pub amount_yuan: f64,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RechargeQuoteQuery {
    /// amount in fen
    pub amount: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RechargeQuoteResp {
    pub amount: i64,
    pub amount_yuan: f64,
    pub estimated_paid_amount: i64,
    pub estimated_paid_amount_yuan: f64,
    pub reimbursement_rate: f64,
    pub message: String,
}

pub async fn quote_recharge(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<RechargeQuoteQuery>,
) -> Result<Json<ApiResponse<RechargeQuoteResp>>, AppError> {
    let _openid = validate_wechat_user(&state, &headers).await?;
    validation::validate_recharge_amount(query.amount)?;

    let estimated_paid_amount = query.amount;

    Ok(Json(ApiResponse::success(RechargeQuoteResp {
        amount: query.amount,
        amount_yuan: query.amount as f64 / 100.0,
        estimated_paid_amount,
        estimated_paid_amount_yuan: estimated_paid_amount as f64 / 100.0,
        reimbursement_rate: 1.0,
        message: "充值按输入金额扣款，到账金额等于扣款金额".to_string(),
    })))
}

pub async fn recharge(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<RechargeRequest>,
) -> Result<Json<ApiResponse<RechargeResp>>, AppError> {
    let started = Instant::now();
    let openid = validate_wechat_user(&state, &headers).await?;
    validation::validate_recharge_amount(body.amount)?;
    let user = validation::load_recharge_user(&state, &openid).await?;
    let request_hash = validation::normalize_request_id(&body.request_id)?;

    if let Some(resp) =
        idempotency::resolve_recharge_by_request_id(&state, &openid, &request_hash).await?
    {
        return Ok(Json(ApiResponse::success(resp)));
    }

    let resp = recharge::process_recharge(
        &state,
        &openid,
        &user,
        body.amount,
        &body.payment_password,
        &request_hash,
        started,
    )
    .await?;

    Ok(Json(ApiResponse::success(resp)))
}

/// GET /api/mini/balance - 查询余额和流水
pub async fn get_balance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<BalanceResp>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;
    let resp = balance::load_balance_response(&state, &openid).await?;

    Ok(Json(ApiResponse::success(resp)))
}

#[cfg(test)]
mod tests {
    use super::validation;
    use crate::error::AppError;

    #[test]
    fn request_id_is_trimmed() {
        assert_eq!(
            validation::normalize_request_id("  req-123  ").expect("request id"),
            "req-123"
        );
    }

    #[test]
    fn empty_request_id_is_rejected() {
        let err = validation::normalize_request_id("   ").expect_err("request id");
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}
