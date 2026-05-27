use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::{
    common::api::{ApiResponse, ApiResult},
    domains::catalog::{dto::ServiceCatalogData, service},
    infra::state::AppState,
};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ServiceCatalogQuery {
    pub region_code: Option<String>,
}

#[allow(dead_code)]
pub async fn list_service_items(
    State(state): State<AppState>,
    Query(query): Query<ServiceCatalogQuery>,
) -> ApiResult<ServiceCatalogData> {
    let data = service::list_service_items(&state, query.region_code).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "获取成功".to_string(),
        data,
    }))
}
