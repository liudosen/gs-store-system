use axum::http::StatusCode;
use tracing::warn;

use crate::common::api::ApiError;

pub fn internal_error(error: impl std::fmt::Display) -> ApiError {
    warn!("internal api error: {error}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "服务暂时不可用，请稍后重试",
    )
}
