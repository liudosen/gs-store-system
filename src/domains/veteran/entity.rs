#[derive(sqlx::FromRow)]
pub struct VeteranProfileRow {
    pub id: i64,
}

#[derive(sqlx::FromRow)]
pub struct VeteranProfileDetailRow {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub veteran_card_number: String,
    pub region_code: String,
    pub region_name: String,
    pub service_tags: String,
    pub is_dispatch_ready: i8,
    pub service_status: String,
    pub completed_order_count: i32,
}
