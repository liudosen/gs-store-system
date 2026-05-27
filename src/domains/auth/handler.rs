use axum::{extract::State, Json};

use crate::{
    common::api::{ApiResponse, ApiResult},
    domains::auth::{
        dto::{
            RegisterResultData, RegisterWithSmsCodeRequest, SendSmsCodeData, SendSmsCodeRequest,
        },
        service,
    },
    infra::state::AppState,
};

pub async fn send_sms_code(
    State(state): State<AppState>,
    Json(payload): Json<SendSmsCodeRequest>,
) -> ApiResult<SendSmsCodeData> {
    let data = service::send_sms_code(&state, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "验证码已发送".to_string(),
        data,
    }))
}

pub async fn register_by_sms_code(
    State(state): State<AppState>,
    Json(payload): Json<RegisterWithSmsCodeRequest>,
) -> ApiResult<RegisterResultData> {
    let data = service::register_by_sms_code(&state, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "登录成功".to_string(),
        data,
    }))
}
