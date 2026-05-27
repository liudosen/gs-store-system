use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UpdateVeteranRegionRequest {
    pub region_code: String,
    pub region_name: Option<String>,
}

#[derive(Serialize)]
pub struct VeteranMeData {
    pub profile: VeteranProfileData,
}

#[derive(Serialize)]
pub struct VeteranProfileData {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub veteran_card_number: String,
    pub region_code: String,
    pub region_name: String,
    pub service_tags: Vec<String>,
    pub is_dispatch_ready: bool,
    pub service_status: String,
    pub completed_order_count: i32,
}

#[derive(Serialize)]
pub struct VeteranStatsData {
    pub today_orders: i64,
    pub today_completed: i64,
    pub month_orders: i64,
    pub rating_score: f64,
}
