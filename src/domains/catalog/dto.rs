use serde::Serialize;

#[derive(Serialize)]
pub struct ServiceCatalogData {
    pub items: Vec<ServiceItemData>,
    pub time_slots: Vec<String>,
}

#[derive(Serialize)]
pub struct ServiceItemData {
    pub id: i64,
    pub code: String,
    pub category_name: String,
    pub name: String,
    pub short_description: String,
    pub badge: String,
    pub base_price: String,
    pub duration_minutes: i32,
}
