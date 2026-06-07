use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ─── DB row structs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct OrderRow {
    pub id: u64,
    pub order_no: String,
    pub external_order_no: Option<String>,
    pub user_id: u64,
    pub address_id: Option<u64>,
    pub status: i8,
    pub total_amount: i64,
    pub paid_amount: i64,
    pub discount_amount: i64,
    pub remark: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct OrderItemRow {
    pub id: u64,
    pub order_id: u64,
    pub order_no: String,
    pub spu_id: u64,
    pub sku_id: u64,
    pub is_default_sku: bool,
    pub goods_title: String,
    pub goods_image: String,
    pub spec_info: String, // JSON
    pub unit_price: i64,
    pub quantity: i32,
    pub subtotal: i64,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct OrderLogisticsRow {
    pub id: u64,
    pub order_id: u64,
    pub order_no: String,
    pub carrier: String,
    pub tracking_no: String,
    pub delivery_name: String,
    pub delivery_phone: String,
    pub remark: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 地址快照，内嵌到 OrderResp 中（读取下单时的地址记录）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderAddressSnap {
    pub id: String,
    pub receiver_name: String,
    pub phone: String,
    pub province: String,
    pub city: String,
    pub district: String,
    pub detail_address: String,
    pub label: String,
}

// ─── Order status ─────────────────────────────────────────────────────────────

pub fn status_label(status: i8) -> &'static str {
    match status {
        0 => "待付款",
        1 => "待发货",
        2 => "待收货",
        3 => "已完成",
        4 => "已取消",
        _ => "未知",
    }
}

// ─── Mini-app request types ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderItemReq {
    pub sku_id: Option<String>,
    pub spu_id: Option<String>,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    pub items: Vec<CreateOrderItemReq>,
    pub address_id: String,
    pub remark: Option<String>,
}

// ─── API response types ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemResp {
    pub id: String,
    pub spu_id: String,
    pub sku_id: Option<String>,
    pub goods_title: String,
    pub goods_image: String,
    pub spec_info: serde_json::Value,
    pub unit_price: i64,
    pub quantity: i32,
    pub subtotal: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderLogisticsResp {
    pub id: String,
    pub order_id: String,
    pub order_no: String,
    pub carrier: String,
    pub tracking_no: String,
    pub delivery_name: String,
    pub delivery_phone: String,
    pub remark: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResp {
    pub id: String,
    pub order_no: String,
    pub external_order_no: Option<String>,
    pub openid: String,
    pub status: i8,
    pub status_label: String,
    pub total_amount: i64,
    pub paid_amount: i64,
    pub discount_amount: i64,
    pub remark: Option<String>,
    pub address: Option<OrderAddressSnap>,
    pub logistics: Option<OrderLogisticsResp>,
    pub items: Vec<OrderItemResp>,
    pub created_at: String,
    pub updated_at: String,
}

/// 支付订单请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayOrderRequest {
    pub payment_password: String,
}

#[derive(Debug, Deserialize)]
pub struct BalancePayRequest {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalancePayResp {
    pub success: bool,
    pub paid_amount: i64,
    pub balance_after: i64,
    pub order_status: Option<i64>,
    pub message: String,
}

// ─── Admin request types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: i8,
    #[serde(alias = "totalAmount")]
    pub total_amount: Option<i64>,
    pub carrier: Option<String>,
    #[serde(alias = "trackingNo")]
    pub tracking_no: Option<String>,
    #[serde(alias = "deliveryName")]
    pub delivery_name: Option<String>,
    #[serde(alias = "deliveryPhone")]
    pub delivery_phone: Option<String>,
    #[serde(alias = "logisticsRemark")]
    pub logistics_remark: Option<String>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

pub fn build_order_item_resp(row: &OrderItemRow, hide_default_sku: bool) -> OrderItemResp {
    let spec_info =
        serde_json::from_str(&row.spec_info).unwrap_or(serde_json::Value::Array(vec![]));
    OrderItemResp {
        id: row.id.to_string(),
        spu_id: row.spu_id.to_string(),
        sku_id: if hide_default_sku && row.is_default_sku {
            None
        } else {
            Some(row.sku_id.to_string())
        },
        goods_title: row.goods_title.clone(),
        goods_image: row.goods_image.clone(),
        spec_info,
        unit_price: row.unit_price,
        quantity: row.quantity,
        subtotal: row.subtotal,
    }
}

pub fn build_order_resp(
    order: &OrderRow,
    items: Vec<OrderItemRow>,
    address: Option<OrderAddressSnap>,
    logistics: Option<OrderLogisticsRow>,
    openid: String,
    hide_default_sku: bool,
) -> OrderResp {
    OrderResp {
        id: order.id.to_string(),
        order_no: order.order_no.clone(),
        external_order_no: order.external_order_no.clone(),
        openid,
        status: order.status,
        status_label: status_label(order.status).to_string(),
        total_amount: order.total_amount,
        paid_amount: order.paid_amount,
        discount_amount: order.discount_amount,
        remark: order.remark.clone(),
        address,
        logistics: logistics.map(|row| OrderLogisticsResp {
            id: row.id.to_string(),
            order_id: row.order_id.to_string(),
            order_no: row.order_no,
            carrier: row.carrier,
            tracking_no: row.tracking_no,
            delivery_name: row.delivery_name,
            delivery_phone: row.delivery_phone,
            remark: row.remark,
            created_at: row.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: row.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }),
        items: items
            .iter()
            .map(|item| build_order_item_resp(item, hide_default_sku))
            .collect(),
        created_at: order.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: order.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}
