use crate::error::AppError;
use crate::routes::mini_app::auth::validate_wechat_user;
use crate::routes::ApiResponse;
use crate::services::{account, jk_pay};
use crate::state::AppState;
use axum::{extract::State, http::HeaderMap, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCardBalanceRequest {
    pub payment_password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCardBalanceResp {
    pub real_name: String,
    pub id_card_masked: String,
    pub balance_fen: i64,
    pub balance_yuan: String,
    pub queried_at: String,
}

fn mask_id_card_number(id_card_number: &str) -> String {
    let chars: Vec<char> = id_card_number.chars().collect();
    if chars.len() <= 8 {
        return id_card_number.to_string();
    }

    let prefix: String = chars.iter().take(4).copied().collect();
    let suffix: String = chars.iter().skip(chars.len() - 4).copied().collect();
    format!("{prefix}********{suffix}")
}

pub async fn get_my_health_card_balance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<HealthCardBalanceRequest>,
) -> Result<Json<ApiResponse<HealthCardBalanceResp>>, AppError> {
    let openid = validate_wechat_user(&state, &headers).await?;

    let payment_password = body.payment_password.trim().to_string();
    if payment_password.is_empty() {
        return Err(AppError::BadRequest(
            "请输入支付密码后再查询健康卡余额".to_string(),
        ));
    }

    let (real_name, id_card_number) =
        account::real_name_and_id_card_number(&state, &openid).await?;

    if real_name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "请先完成认证号认证后再查询健康卡余额".to_string(),
        ));
    }

    if id_card_number.trim().is_empty() {
        return Err(AppError::BadRequest(
            "请先绑定认证号后再查询健康卡余额".to_string(),
        ));
    }

    let mut redis_conn = state.redis_conn().await?;
    let balance_fen = jk_pay::query_health_card_balance(
        &mut redis_conn,
        &state.jk_seller_username,
        &state.jk_seller_password,
        &id_card_number,
        &payment_password,
    )
    .await
    .map_err(AppError::BadRequest)?;

    // 查询成功后回填用户输入的支付密码，便于后续支付/查询复用
    sqlx::query("UPDATE wechat_users SET payment_password = ? WHERE openid = ?")
        .bind(&payment_password)
        .bind(&openid)
        .execute(&state.db)
        .await?;

    let balance_yuan = balance_fen as f64 / 100.0;
    let queried_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    Ok(Json(ApiResponse::success(HealthCardBalanceResp {
        real_name,
        id_card_masked: mask_id_card_number(&id_card_number),
        balance_fen: balance_fen.round() as i64,
        balance_yuan: format!("{balance_yuan:.2}"),
        queried_at,
    })))
}
