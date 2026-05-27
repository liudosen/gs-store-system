use axum::http::{HeaderMap, StatusCode};

use crate::{
    common::api::ApiError,
    domains::{
        auth::service as auth_service,
        veteran::{
            dto::{UpdateVeteranRegionRequest, VeteranMeData, VeteranProfileData},
            repository,
        },
    },
    infra::state::AppState,
};

pub async fn get_me(state: &AppState, headers: &HeaderMap) -> Result<VeteranMeData, ApiError> {
    let session = auth_service::require_veteran_session(state, headers).await?;
    let profile = repository::find_veteran_profile_by_id(&state.db, session.veteran_id)
        .await?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid session"))?;

    Ok(VeteranMeData {
        profile: VeteranProfileData {
            id: profile.id,
            name: profile.name,
            phone: profile.phone,
            veteran_card_number: profile.veteran_card_number,
            region_code: profile.region_code,
            region_name: profile.region_name,
            service_tags: profile
                .service_tags
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect(),
            is_dispatch_ready: profile.is_dispatch_ready == 1,
            service_status: profile.service_status,
            completed_order_count: profile.completed_order_count,
        },
    })
}

pub async fn get_daily_stats(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<super::dto::VeteranStatsData, ApiError> {
    let session = auth_service::require_veteran_session(state, headers).await?;
    let (today_orders, today_completed, month_orders, rating_score) =
        repository::get_daily_stats(&state.db, session.veteran_id).await?;

    Ok(super::dto::VeteranStatsData {
        today_orders,
        today_completed,
        month_orders,
        rating_score,
    })
}

pub async fn update_region(
    state: &AppState,
    headers: &HeaderMap,
    payload: UpdateVeteranRegionRequest,
) -> Result<VeteranMeData, ApiError> {
    let session = auth_service::require_veteran_session(state, headers).await?;
    let region_code = payload.region_code.trim();
    if region_code.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "region code is required"));
    }

    let region_name = payload
        .region_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(region_code);

    repository::update_veteran_region(&state.db, session.veteran_id, region_code, region_name)
        .await?;

    get_me(state, headers).await
}
