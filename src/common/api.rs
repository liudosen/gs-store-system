use axum::{http::StatusCode, Json};
use serde::Serialize;

pub type ApiError = (StatusCode, &'static str);
pub type ApiResult<T> = Result<Json<ApiResponse<T>>, ApiError>;

#[derive(Serialize)]
pub struct Health {
    pub status: &'static str,
    pub app: &'static str,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: T,
}
