use axum::http::StatusCode;

use crate::{
    common::{api::ApiError, phone::normalize_phone},
    domains::{
        onboarding::dto::{VeteranJoinData, VeteranJoinRequest},
        veteran::repository,
    },
    infra::state::AppState,
};

pub async fn veteran_join(
    state: &AppState,
    payload: VeteranJoinRequest,
) -> Result<VeteranJoinData, ApiError> {
    let phone = normalize_phone(payload.phone)?;
    let name = payload.name.trim();
    let id_number = payload.id_number.trim();
    let veteran_card_number = payload.veteran_card_number.trim();

    if name.is_empty() || id_number.is_empty() || veteran_card_number.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "请完整填写入驻资料"));
    }

    let veteran_id = repository::upsert_veteran_profile(
        &state.db,
        repository::UpsertVeteranProfileParams {
            name,
            id_number,
            phone: &phone,
            veteran_card_number,
        },
    )
    .await?;

    Ok(VeteranJoinData { veteran_id, phone })
}
