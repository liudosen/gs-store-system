use axum::{extract::State, http::HeaderMap, Json};

use crate::{
    common::api::{ApiResponse, ApiResult},
    domains::veteran::{dto::{UpdateVeteranRegionRequest, VeteranMeData}, service},
    infra::state::AppState,
};

pub async fn get_me(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<VeteranMeData> {
    let data = service::get_me(&state, &headers).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "ok".to_string(),
        data,
    }))
}

pub async fn get_daily_stats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<super::dto::VeteranStatsData> {
    let data = service::get_daily_stats(&state, &headers).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "ok".to_string(),
        data,
    }))
}

pub async fn update_region(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateVeteranRegionRequest>,
) -> ApiResult<VeteranMeData> {
    let data = service::update_region(&state, &headers, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "region updated".to_string(),
        data,
    }))
}
