#[derive(Clone, sqlx::FromRow)]
pub struct ServiceItemRow {
    pub id: i64,
    pub code: String,
    pub category_name: String,
    pub name: String,
    pub short_description: String,
    pub badge: String,
    pub base_price: String,
    pub duration_minutes: i32,
    pub visible_in_customer: i8,
}
