use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct VeteranJoinRequest {
    pub name: String,
    pub id_number: String,
    pub phone: String,
    pub veteran_card_number: String,
}

#[derive(Serialize)]
pub struct VeteranJoinData {
    pub veteran_id: i64,
    pub phone: String,
}
