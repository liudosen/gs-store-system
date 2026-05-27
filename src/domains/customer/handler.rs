use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};

use crate::{
    common::api::{ApiResponse, ApiResult},
    domains::{
        catalog::dto::ServiceCatalogData,
        customer::{
            dto::{
                AddressListData, CustomerLoginBySmsRequest, CustomerLoginData, CustomerMeData,
                CustomerSmsCodeData, SendCustomerSmsCodeRequest, UpdateRegionRequest,
                UpsertAddressData, UpsertAddressRequest,
            },
            service,
        },
    },
    infra::state::AppState,
};

#[derive(serde::Deserialize)]
pub struct CustomerServiceCatalogQuery {
    pub region_code: Option<String>,
}

pub async fn send_sms_code(
    State(state): State<AppState>,
    Json(payload): Json<SendCustomerSmsCodeRequest>,
) -> ApiResult<CustomerSmsCodeData> {
    let data = service::send_sms_code(&state, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "验证码已发送".to_string(),
        data,
    }))
}

pub async fn login_by_sms(
    State(state): State<AppState>,
    Json(payload): Json<CustomerLoginBySmsRequest>,
) -> ApiResult<CustomerLoginData> {
    let data = service::login_by_sms(&state, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "登录成功".to_string(),
        data,
    }))
}

pub async fn get_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<CustomerMeData> {
    let data = service::get_me(&state, &headers).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "获取成功".to_string(),
        data,
    }))
}

pub async fn update_region(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateRegionRequest>,
) -> ApiResult<CustomerMeData> {
    let data = service::update_region(&state, &headers, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "地区已更新".to_string(),
        data,
    }))
}

pub async fn list_addresses(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<AddressListData> {
    let data = service::list_addresses(&state, &headers).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "获取成功".to_string(),
        data,
    }))
}

pub async fn create_address(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpsertAddressRequest>,
) -> ApiResult<UpsertAddressData> {
    let data = service::create_address(&state, &headers, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "地址已保存".to_string(),
        data,
    }))
}

pub async fn update_address(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(address_id): Path<i64>,
    Json(payload): Json<UpsertAddressRequest>,
) -> ApiResult<UpsertAddressData> {
    let data = service::update_address(&state, &headers, address_id, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "地址已更新".to_string(),
        data,
    }))
}

pub async fn list_service_items(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CustomerServiceCatalogQuery>,
) -> ApiResult<ServiceCatalogData> {
    let data = service::list_service_items(&state, &headers, query.region_code).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "ok".to_string(),
        data,
    }))
}
