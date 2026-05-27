use axum::http::{HeaderMap, StatusCode};
use rand::{thread_rng, Rng};
use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    common::{api::ApiError, errors::internal_error, phone::normalize_phone},
    domains::{
        catalog::service as catalog_service,
        customer::{
            dto::{
                AddressData, AddressListData, CustomerLoginBySmsRequest, CustomerLoginData,
                CustomerMeData, CustomerProfileData, CustomerSmsCodeData, RegionData,
                SendCustomerSmsCodeRequest, UpdateRegionRequest, UpsertAddressData,
                UpsertAddressRequest,
            },
            entity::{CustomerAddressRow, CustomerUserRow, ServiceRegionRow},
            repository,
        },
    },
    infra::state::AppState,
};

const SMS_CODE_TTL_SECONDS: u64 = 300;
const SMS_SEND_INTERVAL_SECONDS: u64 = 60;
const CUSTOMER_SESSION_TTL_SECONDS: u64 = 60 * 60 * 24 * 30;

#[derive(Clone)]
pub struct CustomerSession {
    pub user_id: i64,
}

pub async fn send_sms_code(
    state: &AppState,
    payload: SendCustomerSmsCodeRequest,
) -> Result<CustomerSmsCodeData, ApiError> {
    let phone = normalize_phone(payload.phone)?;
    let lock_key = sms_lock_key(&phone);
    let code_key = sms_code_key(&phone);
    let mut redis = state.redis.clone();

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

    Ok(CustomerSmsCodeData {
        expires_in_seconds: SMS_CODE_TTL_SECONDS,
        next_send_in_seconds: SMS_SEND_INTERVAL_SECONDS,
    })
}

pub async fn login_by_sms(
    state: &AppState,
    payload: CustomerLoginBySmsRequest,
) -> Result<CustomerLoginData, ApiError> {
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

    let user = repository::create_or_update_customer_user(
        &state.db,
        repository::CreateOrUpdateCustomerUserParams { phone: &phone },
    )
    .await?;

    let token = Uuid::new_v4().to_string();
    let _: () = redis_set_with_retry(
        &mut redis,
        &session_key(&token),
        &format!("{}|{}", user.id, user.phone),
        CUSTOMER_SESSION_TTL_SECONDS,
    )
    .await
    .map_err(internal_error)?;
    let _: () = redis
        .del(sms_code_key(&phone))
        .await
        .map_err(internal_error)?;

    Ok(CustomerLoginData {
        user_id: user.id,
        phone,
        token,
    })
}

pub async fn get_me(state: &AppState, headers: &HeaderMap) -> Result<CustomerMeData, ApiError> {
    let session = require_customer_session(state, headers).await?;
    let user = repository::find_customer_user_by_id(&state.db, session.user_id)
        .await?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid session"))?;
    let regions = repository::list_regions(&state.db).await?;

    Ok(CustomerMeData {
        user: map_user(user),
        regions: regions.into_iter().map(map_region).collect(),
    })
}

pub async fn list_service_items(
    state: &AppState,
    headers: &HeaderMap,
    region_code: Option<String>,
) -> Result<crate::domains::catalog::dto::ServiceCatalogData, ApiError> {
    let _ = require_customer_session(state, headers).await?;
    catalog_service::list_service_items(state, region_code).await
}

pub async fn update_region(
    state: &AppState,
    headers: &HeaderMap,
    payload: UpdateRegionRequest,
) -> Result<CustomerMeData, ApiError> {
    let session = require_customer_session(state, headers).await?;
    let region = repository::find_region_by_code(&state.db, payload.region_code.trim())
        .await?
        .ok_or((StatusCode::BAD_REQUEST, "unknown region"))?;

    repository::update_customer_region(
        &state.db,
        repository::UpdateRegionParams {
            user_id: session.user_id,
            region_code: &region.code,
            region_name: &region.name,
        },
    )
    .await?;

    get_me(state, headers).await
}

pub async fn list_addresses(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AddressListData, ApiError> {
    let session = require_customer_session(state, headers).await?;
    let rows = repository::list_addresses_by_user_id(&state.db, session.user_id).await?;

    Ok(AddressListData {
        items: rows.into_iter().map(map_address).collect(),
    })
}

pub async fn create_address(
    state: &AppState,
    headers: &HeaderMap,
    payload: UpsertAddressRequest,
) -> Result<UpsertAddressData, ApiError> {
    save_address(state, headers, None, payload).await
}

pub async fn update_address(
    state: &AppState,
    headers: &HeaderMap,
    address_id: i64,
    payload: UpsertAddressRequest,
) -> Result<UpsertAddressData, ApiError> {
    save_address(state, headers, Some(address_id), payload).await
}

async fn save_address(
    state: &AppState,
    headers: &HeaderMap,
    address_id: Option<i64>,
    payload: UpsertAddressRequest,
) -> Result<UpsertAddressData, ApiError> {
    let session = require_customer_session(state, headers).await?;
    let region_code = payload.region_code.trim().to_string();
    let region_name = payload.region_name.trim().to_string();
    let city_name = payload.city_name.trim().to_string();
    let district_name = payload.district_name.trim().to_string();
    let detail_address = payload.detail_address.trim().to_string();
    let contact_name = payload.contact_name.trim().to_string();
    let contact_phone = normalize_phone(payload.contact_phone)?;

    if region_code.is_empty()
        || region_name.is_empty()
        || city_name.is_empty()
        || district_name.is_empty()
        || detail_address.is_empty()
        || contact_name.is_empty()
    {
        return Err((StatusCode::BAD_REQUEST, "incomplete address payload"));
    }

    let address = repository::upsert_address(
        &state.db,
        repository::UpsertAddressParams {
            address_id,
            user_id: session.user_id,
            region_code: &region_code,
            region_name: &region_name,
            city_name: &city_name,
            district_name: &district_name,
            detail_address: &detail_address,
            contact_name: &contact_name,
            contact_phone: &contact_phone,
            is_default: payload.is_default,
        },
    )
    .await?;

    Ok(UpsertAddressData {
        address: map_address(address),
    })
}

pub async fn require_customer_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<CustomerSession, ApiError> {
    let token = extract_bearer_token(headers)?;
    let mut redis = state.redis.clone();
    let value: Option<String> = redis
        .get(session_key(&token))
        .await
        .map_err(internal_error)?;

    let value = value.ok_or((StatusCode::UNAUTHORIZED, "invalid session"))?;
    let mut parts = value.splitn(2, '|');
    let user_id = parts
        .next()
        .and_then(|item| item.parse::<i64>().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "invalid session"))?;

    Ok(CustomerSession { user_id })
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
    format!("sms:code:customer:{phone}")
}

fn sms_lock_key(phone: &str) -> String {
    format!("sms:lock:customer:{phone}")
}

fn session_key(token: &str) -> String {
    format!("session:customer:{token}")
}

fn map_region(region: ServiceRegionRow) -> RegionData {
    RegionData {
        code: region.code,
        name: region.name,
        city_name: region.city_name,
        district_name: region.district_name,
    }
}

fn map_user(user: CustomerUserRow) -> CustomerProfileData {
    CustomerProfileData {
        id: user.id,
        phone: user.phone,
        selected_region_code: user.selected_region_code,
        selected_region_name: user.selected_region_name,
    }
}

pub fn map_address(address: CustomerAddressRow) -> AddressData {
    let _ = address.user_id;

    AddressData {
        id: address.id,
        region_code: address.region_code,
        region_name: address.region_name,
        city_name: address.city_name,
        district_name: address.district_name,
        detail_address: address.detail_address,
        contact_name: address.contact_name,
        contact_phone: address.contact_phone,
        is_default: address.is_default == 1,
    }
}

// ---- Redis 重试辅助 ----

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
