use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SendSmsCodeRequest {
    pub phone: String,
}

#[derive(Deserialize)]
pub struct RegisterWithSmsCodeRequest {
    pub phone: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct SendSmsCodeData {
    pub expires_in_seconds: u64,
    pub next_send_in_seconds: u64,
}

#[derive(Serialize)]
pub struct RegisterResultData {
    pub veteran_id: i64,
    pub phone: String,
    pub auto_registered: bool,
    pub token: String,
    pub expires_in_seconds: u64,
}
