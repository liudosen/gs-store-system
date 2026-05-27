use axum::http::StatusCode;

use crate::common::api::ApiError;

pub fn normalize_phone(phone: String) -> Result<String, ApiError> {
    let phone = phone.trim().to_string();
    if phone.len() != 11 || !phone.chars().all(|item| item.is_ascii_digit()) {
        return Err((StatusCode::BAD_REQUEST, "手机号格式不正确"));
    }
    Ok(phone)
}
