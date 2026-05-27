#[derive(Clone, sqlx::FromRow)]
pub struct CustomerUserRow {
    pub id: i64,
    pub phone: String,
    pub selected_region_code: Option<String>,
    pub selected_region_name: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
pub struct CustomerAddressRow {
    pub id: i64,
    pub user_id: i64,
    pub region_code: String,
    pub region_name: String,
    pub city_name: String,
    pub district_name: String,
    pub detail_address: String,
    pub contact_name: String,
    pub contact_phone: String,
    pub is_default: i8,
}

#[derive(Clone, sqlx::FromRow)]
pub struct ServiceRegionRow {
    pub code: String,
    pub name: String,
    pub city_name: String,
    pub district_name: String,
}
