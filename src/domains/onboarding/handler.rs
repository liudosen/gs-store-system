use axum::{extract::State, Json};

use crate::{
    common::api::{ApiResponse, ApiResult},
    domains::onboarding::{
        dto::{VeteranJoinData, VeteranJoinRequest},
        service,
    },
    infra::state::AppState,
};

pub async fn veteran_join(
    State(state): State<AppState>,
    Json(payload): Json<VeteranJoinRequest>,
) -> ApiResult<VeteranJoinData> {
    let data = service::veteran_join(&state, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "入驻成功，手机号已注册为就业平台账户".to_string(),
        data,
    }))
}
