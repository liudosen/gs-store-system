use axum::http::{HeaderMap, StatusCode};
use rand::{thread_rng, Rng};
use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    common::{api::ApiError, errors::internal_error, phone::normalize_phone},
    domains::{
        auth::dto::{
            RegisterResultData, RegisterWithSmsCodeRequest, SendSmsCodeData, SendSmsCodeRequest,
        },
        veteran::repository,
    },
    infra::state::AppState,
};

const SMS_CODE_TTL_SECONDS: u64 = 300;
const SMS_SEND_INTERVAL_SECONDS: u64 = 60;
const VETERAN_SESSION_TTL_SECONDS: u64 = 60 * 60 * 24 * 30;

#[derive(Clone)]
pub struct VeteranSession {
    pub veteran_id: i64,
}

pub async fn send_sms_code(
    state: &AppState,
    payload: SendSmsCodeRequest,
) -> Result<SendSmsCodeData, ApiError> {
    let phone = normalize_phone(payload.phone)?;

    if repository::find_veteran_profile_by_phone(&state.db, &phone)
        .await?
        .is_none()
    {
        return Err((StatusCode::FORBIDDEN, "veteran profile not onboarded"));
    }

    let mut redis = state.redis.clone();
    let lock_key = sms_lock_key(&phone);
    let code_key = sms_code_key(&phone);

    let locked: bool = redis.exists(&lock_key).await.unwrap_or(false);
    if locked {
        return Err((StatusCode::TOO_MANY_REQUESTS, "too many sms requests"));
    }

    let code = generate_sms_code();

    // 先存验证码到 Redis（快），再异步发送短信
    let _: () = redis
        .set_ex(&code_key, &code, SMS_CODE_TTL_SECONDS)
        .await
        .map_err(internal_error)?;
    let _: () = redis
        .set_ex(&lock_key, "1", SMS_SEND_INTERVAL_SECONDS)
        .await
        .map_err(internal_error)?;

    // 后台异步发送短信，不阻塞响应
    if let Some(sms_service) = state.sms_service.clone() {
        let phone_clone = phone.clone();
        let code_clone = code.clone();
        tokio::spawn(async move {
            if let Err(e) = sms_service.send_code(&phone_clone, &code_clone).await {
                tracing::warn!(phone = %phone_clone, error = %e, "async sms send failed");
            }
        });
    }

    Ok(SendSmsCodeData {
        expires_in_seconds: SMS_CODE_TTL_SECONDS,
        next_send_in_seconds: SMS_SEND_INTERVAL_SECONDS,
    })
}

pub async fn register_by_sms_code(
    state: &AppState,
    payload: RegisterWithSmsCodeRequest,
) -> Result<RegisterResultData, ApiError> {
    let phone = normalize_phone(payload.phone)?;
    let code = payload.code.trim().to_string();
    if code.len() != 6 || !code.chars().all(|item| item.is_ascii_digit()) {
        return Err((StatusCode::BAD_REQUEST, "invalid sms code format"));
    }

    let mut redis = state.redis.clone();
    let saved_code: Option<String> = redis_get_with_retry(&mut redis, &sms_code_key(&phone))
        .await
        .map_err(internal_error)?;

    match saved_code {
        Some(saved) if saved == code => {}
        Some(_) => return Err((StatusCode::BAD_REQUEST, "invalid sms code")),
        None => return Err((StatusCode::BAD_REQUEST, "sms code expired")),
    }

    let veteran = repository::find_veteran_profile_by_phone(&state.db, &phone)
        .await?
        .ok_or((StatusCode::FORBIDDEN, "veteran profile not onboarded"))?;

    let token = Uuid::new_v4().to_string();
    let _: () = redis_set_with_retry(
        &mut redis,
        &session_key(&token),
        &veteran.id.to_string(),
        VETERAN_SESSION_TTL_SECONDS,
    )
    .await
    .map_err(internal_error)?;

    let _: () = redis
        .del(sms_code_key(&phone))
        .await
        .map_err(internal_error)?;

    Ok(RegisterResultData {
        veteran_id: veteran.id,
        phone,
        auto_registered: false,
        token,
        expires_in_seconds: VETERAN_SESSION_TTL_SECONDS,
    })
}

pub async fn resolve_veteran_token(
    state: &AppState,
    raw_token: &str,
) -> Result<VeteranSession, ApiError> {
    let mut redis = state.redis.clone();
    let value: Option<String> = redis
        .get(session_key(raw_token))
        .await
        .map_err(internal_error)?;

    let veteran_id = value
        .and_then(|item| item.parse::<i64>().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "invalid token"))?;

    Ok(VeteranSession { veteran_id })
}

pub async fn require_veteran_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<VeteranSession, ApiError> {
    let token = extract_bearer_token(headers)?;
    let mut redis = state.redis.clone();
    let value: Option<String> = redis
        .get(session_key(&token))
        .await
        .map_err(internal_error)?;

    let veteran_id = value
        .and_then(|item| item.parse::<i64>().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "invalid session"))?;

    Ok(VeteranSession { veteran_id })
}

// ---- 辅助函数 ----

async fn redis_get_with_retry(
    redis: &mut redis::aio::ConnectionManager,
    key: &str,
) -> Result<Option<String>, redis::RedisError> {
    match redis.get(key).await {
        Ok(val) => Ok(val),
        Err(e) => {
            tracing::warn!(key, error = %e, "redis get failed, retrying once");
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            redis.get(key).await
        }
    }
}

async fn redis_set_with_retry(
    redis: &mut redis::aio::ConnectionManager,
    key: &str,
    value: &str,
    ttl: u64,
) -> Result<(), redis::RedisError> {
    match redis.set_ex(key, value, ttl).await {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::warn!(key, error = %e, "redis set failed, retrying once");
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            redis.set_ex(key, value, ttl).await
        }
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, ApiError> {
    let header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "authorization required"))?;

    let token = header
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "authorization required"))?;

    Ok(token.to_string())
}

fn generate_sms_code() -> String {
    format!("{:06}", thread_rng().gen_range(0..1_000_000))
}

fn sms_code_key(phone: &str) -> String {
    format!("sms:code:register:{phone}")
}

fn sms_lock_key(phone: &str) -> String {
    format!("sms:lock:register:{phone}")
}

fn session_key(token: &str) -> String {
    format!("session:veteran:{token}")
}