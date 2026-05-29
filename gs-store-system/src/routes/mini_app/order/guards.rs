use crate::error::AppError;
use crate::models::order::CreateOrderRequest;
use crate::state::AppState;
use sha2::{Digest, Sha256};
use std::future::Future;

const CREATE_ORDER_DEDUP_SECONDS: usize = 10;
const PAY_ORDER_LOCK_TTL_SECS: usize = 180;

pub(crate) fn build_payment_submit_guard_key(openid: &str) -> String {
    format!("order:pay:{openid}")
}

pub(super) fn build_order_submit_guard_key(openid: &str, fingerprint: &str) -> String {
    format!("order:create:{openid}:{fingerprint}")
}

pub(super) fn build_order_request_fingerprint(body: &CreateOrderRequest) -> String {
    let mut normalized_items: Vec<String> = body
        .items
        .iter()
        .map(|item| {
            let item_id = item
                .sku_id
                .as_deref()
                .map(|sku| format!("sku:{sku}"))
                .or_else(|| item.spu_id.as_deref().map(|spu| format!("spu:{spu}")))
                .unwrap_or_else(|| "missing".to_string());
            format!("{item_id}:{}", item.quantity)
        })
        .collect();
    normalized_items.sort();

    let raw = format!(
        "{}|{}|{}",
        body.address_id,
        body.remark.as_deref().unwrap_or(""),
        normalized_items.join(",")
    );
    hex::encode(Sha256::digest(raw.as_bytes()))
}

pub(crate) async fn try_acquire_payment_submit_guard(
    state: &AppState,
    key: &str,
) -> Result<bool, AppError> {
    let mut redis = state.redis_conn().await?;
    let response: Option<String> = redis::cmd("SET")
        .arg(key)
        .arg("PENDING")
        .arg("NX")
        .arg("EX")
        .arg(PAY_ORDER_LOCK_TTL_SECS)
        .query_async(&mut redis)
        .await
        .map_err(|e| AppError::InternalError(format!("failed to lock payment submission: {e}")))?;
    Ok(response.is_some())
}

pub(crate) async fn clear_payment_submit_guard(state: &AppState, key: &str) {
    let Ok(mut redis) = state.redis_conn().await else {
        return;
    };
    let _ = redis::cmd("DEL")
        .arg(key)
        .query_async::<_, i32>(&mut redis)
        .await;
}

pub(crate) async fn with_payment_submit_guard<T, F>(
    state: &AppState,
    openid: &str,
    fut: F,
) -> Result<T, AppError>
where
    F: Future<Output = Result<T, AppError>>,
{
    let payment_lock_key = build_payment_submit_guard_key(openid);
    if !try_acquire_payment_submit_guard(state, &payment_lock_key).await? {
        return Err(AppError::BadRequest(
            "payment in progress, please retry".to_string(),
        ));
    }

    let result = fut.await;
    clear_payment_submit_guard(state, &payment_lock_key).await;
    result
}

pub(super) async fn try_acquire_order_submit_guard(
    state: &AppState,
    key: &str,
) -> Result<bool, AppError> {
    let mut redis = state.redis_conn().await?;
    let response: Option<String> = redis::cmd("SET")
        .arg(key)
        .arg("PENDING")
        .arg("NX")
        .arg("EX")
        .arg(CREATE_ORDER_DEDUP_SECONDS)
        .query_async(&mut redis)
        .await
        .map_err(|e| AppError::InternalError(format!("failed to lock order submission: {e}")))?;
    Ok(response.is_some())
}

pub(super) async fn load_order_submit_guard(
    state: &AppState,
    key: &str,
) -> Result<Option<String>, AppError> {
    let mut redis = state.redis_conn().await?;
    redis::cmd("GET")
        .arg(key)
        .query_async(&mut redis)
        .await
        .map_err(|e| AppError::InternalError(format!("failed to read order submission lock: {e}")))
}

pub(super) async fn set_order_submit_guard_order_id(
    state: &AppState,
    key: &str,
    order_id: u64,
) -> Result<(), AppError> {
    let mut redis = state.redis_conn().await?;
    let _: () = redis::cmd("SET")
        .arg(key)
        .arg(order_id.to_string())
        .arg("EX")
        .arg(CREATE_ORDER_DEDUP_SECONDS)
        .query_async(&mut redis)
        .await
        .map_err(|e| {
            AppError::InternalError(format!("failed to finalize order submission lock: {e}"))
        })?;
    Ok(())
}

pub(super) async fn clear_order_submit_guard(state: &AppState, key: &str) {
    let Ok(mut redis) = state.redis_conn().await else {
        return;
    };
    let _ = redis::cmd("DEL")
        .arg(key)
        .query_async::<_, i32>(&mut redis)
        .await;
}
