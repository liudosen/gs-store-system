mod guards;
mod lifecycle;
mod mapping;
mod payment;
mod queries;

use crate::error::AppError;
use crate::models::order::{
    BalancePayRequest, BalancePayResp, CreateOrderRequest, OrderResp, PayOrderRequest,
};
use crate::routes::mini_app::auth::validate_wechat_user;
use crate::routes::ApiResponse;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

#[cfg(test)]
pub(crate) use guards::{
    build_payment_submit_guard_key, clear_payment_submit_guard, try_acquire_payment_submit_guard,
    with_payment_submit_guard,
};
#[cfg(test)]
pub(crate) use payment::calc_balance_payment_amount;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOrdersQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub status: Option<i8>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedOrders {
    pub list: Vec<OrderResp>,
    pub total: i64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayOrderResp {
    pub success: bool,
    pub paid_amount: i64,
    pub order_status: Option<i64>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::{
        build_payment_submit_guard_key, calc_balance_payment_amount, clear_payment_submit_guard,
        try_acquire_payment_submit_guard, with_payment_submit_guard, AppError, AppState,
    };
    use sqlx::mysql::MySqlPoolOptions;
    use std::env;

    #[test]
    fn calc_balance_payment_amount_uses_actual_deduct_amount() {
        assert_eq!(calc_balance_payment_amount(100, 0), 105);
        assert_eq!(calc_balance_payment_amount(10_000, 500), 10_000);
        assert_eq!(calc_balance_payment_amount(1, 0), 1);
        assert_eq!(calc_balance_payment_amount(100, 150), 0);
    }

    #[test]
    fn build_payment_submit_guard_key_scopes_by_openid() {
        assert_eq!(
            build_payment_submit_guard_key("openid-123"),
            "order:pay:openid-123"
        );
    }

    fn build_test_redis_pool() -> deadpool_redis::Pool {
        let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let redis_port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
        let redis_username = env::var("REDIS_USERNAME").unwrap_or_else(|_| "default".to_string());
        let redis_password = env::var("REDIS_PASSWORD").unwrap_or_default();
        let redis_url = if redis_password.is_empty() {
            if redis_username == "default" {
                format!("redis://{}:{}/0", redis_host, redis_port)
            } else {
                format!(
                    "redis://{}@{}:{}/0",
                    urlencoding::encode(&redis_username),
                    redis_host,
                    redis_port
                )
            }
        } else {
            format!(
                "redis://{}:{}@{}:{}/0",
                urlencoding::encode(&redis_username),
                urlencoding::encode(&redis_password),
                redis_host,
                redis_port
            )
        };

        let cfg = deadpool_redis::Config::from_url(redis_url);
        cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool")
    }

    fn build_test_state(redis: deadpool_redis::Pool) -> AppState {
        let db = MySqlPoolOptions::new()
            .connect_lazy("mysql://root:root@127.0.0.1:3306/test")
            .expect("lazy mysql pool");

        AppState {
            db,
            redis,
            jwt_secret: "test-secret".to_string(),
            jwt_expiry_hours: 24,
            auth_require_redis_session: false,
            bcrypt_cost: 12,
            wechat_appid: "appid".to_string(),
            wechat_secret: "secret".to_string(),
            dev_wechat_openid: Some("openid-123".to_string()),
            jk_seller_username: "seller".to_string(),
            jk_seller_password: "password".to_string(),
            oss_endpoint: "endpoint".to_string(),
            oss_access_key_id: "ak".to_string(),
            oss_access_key_secret: "sk".to_string(),
            oss_bucket: "bucket".to_string(),
            oss_domain: "domain".to_string(),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn payment_submit_guard_blocks_duplicate_acquire_and_releases_after_use() {
        let redis = build_test_redis_pool();
        let state = build_test_state(redis);
        let openid = "openid-payment-lock-test";
        let lock_key = build_payment_submit_guard_key(openid);

        clear_payment_submit_guard(&state, &lock_key).await;

        assert!(try_acquire_payment_submit_guard(&state, &lock_key)
            .await
            .expect("first acquire should work"));
        assert!(!try_acquire_payment_submit_guard(&state, &lock_key)
            .await
            .expect("second acquire should be blocked"));

        clear_payment_submit_guard(&state, &lock_key).await;

        let result = with_payment_submit_guard(&state, openid, async { Ok::<_, AppError>(42) })
            .await
            .expect("wrapper should succeed");
        assert_eq!(result, 42);

        assert!(try_acquire_payment_submit_guard(&state, &lock_key)
            .await
            .expect("lock should be released after wrapper"));

        clear_payment_submit_guard(&state, &lock_key).await;
    }

    #[tokio::test]
    #[ignore]
    async fn payment_submit_guard_releases_even_when_closure_fails() {
        let redis = build_test_redis_pool();
        let state = build_test_state(redis);
        let openid = "openid-payment-lock-test-fail";
        let lock_key = build_payment_submit_guard_key(openid);

        clear_payment_submit_guard(&state, &lock_key).await;

        let result = with_payment_submit_guard(&state, openid, async {
            Err::<(), AppError>(AppError::BadRequest("boom".to_string()))
        })
        .await;
        assert!(result.is_err(), "closure should fail");

        assert!(try_acquire_payment_submit_guard(&state, &lock_key)
            .await
            .expect("lock should be released after failure"));

        clear_payment_submit_guard(&state, &lock_key).await;
    }
}

pub async fn pay_order_with_balance(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
    Json(body): Json<BalancePayRequest>,
) -> Result<Json<ApiResponse<BalancePayResp>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;
    payment::pay_order_with_balance_impl(&state, &openid, id, body).await
}

pub async fn confirm_my_order_received(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
) -> Result<Json<ApiResponse<OrderResp>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;
    lifecycle::confirm_my_order_received_impl(&state, &openid, id).await
}

pub async fn create_order(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateOrderRequest>,
) -> Result<Json<ApiResponse<OrderResp>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;
    lifecycle::create_order_impl(&state, &openid, body).await
}

pub async fn list_my_orders(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(q): Query<ListOrdersQuery>,
) -> Result<Json<ApiResponse<PagedOrders>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;
    lifecycle::list_my_orders_impl(&state, &openid, q).await
}

pub async fn get_my_order(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
) -> Result<Json<ApiResponse<OrderResp>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;
    lifecycle::get_my_order_impl(&state, &openid, id).await
}

pub async fn cancel_my_order(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
) -> Result<Json<ApiResponse<OrderResp>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;
    lifecycle::cancel_my_order_impl(&state, &openid, id).await
}

pub async fn pay_order(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
    Json(body): Json<PayOrderRequest>,
) -> Result<Json<ApiResponse<PayOrderResp>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;
    payment::pay_order_impl(&state, &openid, id, body).await
}
