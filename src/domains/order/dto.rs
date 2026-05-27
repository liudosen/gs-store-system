use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub service_item_id: i64,
    pub address_id: i64,
    pub service_date: String,
    pub service_time_slot: String,
    pub note: String,
}

#[derive(Deserialize, Clone)]
pub struct CustomerOrderListQuery {
    pub category: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct OrderListData {
    pub items: Vec<OrderData>,
    pub has_more: bool,
    pub next_offset: i64,
}

#[derive(Serialize)]
pub struct CreateOrderData {
    pub order: OrderData,
}

#[derive(Serialize)]
pub struct OrderDetailData {
    pub order: OrderData,
}

#[derive(Clone, Serialize)]
pub struct OrderData {
    pub id: i64,
    pub order_no: String,
    pub status: String,
    pub status_label: String,
    pub service_item_id: i64,
    pub service_item_name: String,
    pub region_name: String,
    pub service_date: String,
    pub service_time_slot: String,
    pub service_address: String,
    pub contact_name: String,
    pub contact_phone: String,
    pub note: String,
    pub assigned_veteran_name: Option<String>,
    pub assigned_veteran_phone: Option<String>,
    pub dispatch_message: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct AvailableOrderListData {
    pub items: Vec<OrderData>,
}

#[derive(Serialize)]
pub struct AcceptOrderData {
    pub order: OrderData,
}

#[derive(Serialize)]
pub struct AssignedOrderListData {
    pub items: Vec<OrderData>,
}
