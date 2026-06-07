use crate::error::AppError;
use crate::models::order::{
    build_order_resp, OrderAddressSnap, OrderItemRow, OrderLogisticsRow, OrderResp, OrderRow,
    UpdateOrderStatusRequest,
};
use crate::routes::admin::auth::authorize_admin;
use crate::routes::admin::permissions::ORDER_LIST_VIEW;
use crate::routes::ApiResponse;
use crate::services::inventory;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ListOrdersQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub status: Option<i8>,
    pub user_id: Option<u64>,
    pub order_no: Option<String>,
    pub openid: Option<String>,
}

#[derive(serde::Serialize)]
pub struct PagedOrders {
    pub list: Vec<OrderResp>,
    pub total: i64,
    pub page: u64,
    pub page_size: u64,
}

async fn fetch_order_items(state: &AppState, order_id: u64) -> Result<Vec<OrderItemRow>, AppError> {
    let items = sqlx::query_as::<_, OrderItemRow>(
        "SELECT oi.id, oi.order_id, oi.order_no, oi.spu_id, oi.sku_id, COALESCE(gs.is_default, 0) AS is_default_sku, \
         oi.goods_title, oi.goods_image, oi.spec_info, oi.unit_price, oi.quantity, oi.subtotal \
         FROM order_items oi LEFT JOIN goods_skus gs ON gs.id = oi.sku_id WHERE oi.order_id = ? ORDER BY oi.id",
    )
    .bind(order_id)
    .fetch_all(&state.db)
    .await?;
    Ok(items)
}

#[derive(sqlx::FromRow)]
struct AddressRow {
    id: u64,
    receiver_name: String,
    phone: String,
    province: String,
    city: String,
    district: String,
    detail_address: String,
    label: String,
}

async fn fetch_address_snap(state: &AppState, address_id: Option<u64>) -> Option<OrderAddressSnap> {
    let id = address_id?;
    sqlx::query_as::<_, AddressRow>(
        "SELECT id, receiver_name, phone, province, city, district, detail_address, label \
         FROM addresses WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .map(|r| OrderAddressSnap {
        id: r.id.to_string(),
        receiver_name: r.receiver_name,
        phone: r.phone,
        province: r.province,
        city: r.city,
        district: r.district,
        detail_address: r.detail_address,
        label: r.label,
    })
}

async fn fetch_order_logistics(
    state: &AppState,
    order_id: u64,
) -> Result<Option<OrderLogisticsRow>, AppError> {
    let logistics = sqlx::query_as::<_, OrderLogisticsRow>(
        "SELECT id, order_id, order_no, carrier, tracking_no, delivery_name, delivery_phone, remark, created_at, updated_at \
         FROM order_logistics WHERE order_id = ?",
    )
    .bind(order_id)
    .fetch_optional(&state.db)
    .await?;
    Ok(logistics)
}

const ORDER_SELECT: &str =
    "SELECT id, order_no, external_order_no, user_id, address_id, status, total_amount, paid_amount, \
     discount_amount, remark, created_at, updated_at FROM orders";

async fn fetch_openid(state: &AppState, user_id: u64) -> String {
    sqlx::query_scalar::<_, String>("SELECT openid FROM wechat_users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
}

pub async fn list_orders(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(q): Query<ListOrdersQuery>,
) -> Result<Json<ApiResponse<PagedOrders>>, AppError> {
    authorize_admin(&state, &headers, &[ORDER_LIST_VIEW]).await?;

    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let mut conditions = vec!["1=1"];
    if q.status.is_some() {
        conditions.push("status = ?");
    }
    if q.user_id.is_some() {
        conditions.push("user_id = ?");
    }
    if q.order_no.is_some() {
        conditions.push("order_no LIKE ?");
    }
    if q.openid.is_some() {
        conditions.push("user_id IN (SELECT id FROM wechat_users WHERE openid LIKE ?)");
    }
    let where_clause = conditions.join(" AND ");

    let count_sql = format!("SELECT COUNT(*) FROM orders WHERE {}", where_clause);
    let list_sql = format!(
        "{} WHERE {} ORDER BY id DESC LIMIT ? OFFSET ?",
        ORDER_SELECT, where_clause
    );

    let q_status = q.status;
    let q_user_id = q.user_id;
    let q_order_no = q.order_no.clone();
    let q_openid = q.openid.clone();

    let mut count_q = sqlx::query_scalar(&count_sql);
    if let Some(st) = q_status {
        count_q = count_q.bind(st);
    }
    if let Some(uid) = q_user_id {
        count_q = count_q.bind(uid);
    }
    if let Some(ref no) = q_order_no {
        count_q = count_q.bind(format!("%{}%", no));
    }
    if let Some(ref openid) = q_openid {
        count_q = count_q.bind(format!("%{}%", openid));
    }
    let total: i64 = count_q.fetch_one(&state.db).await?;

    let mut list_q = sqlx::query_as::<_, OrderRow>(&list_sql);
    if let Some(st) = q_status {
        list_q = list_q.bind(st);
    }
    if let Some(uid) = q_user_id {
        list_q = list_q.bind(uid);
    }
    if let Some(ref no) = q_order_no {
        list_q = list_q.bind(format!("%{}%", no));
    }
    if let Some(ref openid) = q_openid {
        list_q = list_q.bind(format!("%{}%", openid));
    }
    let rows: Vec<OrderRow> = list_q
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    let mut list = Vec::with_capacity(rows.len());
    for row in &rows {
        let items = fetch_order_items(&state, row.id).await?;
        let address = fetch_address_snap(&state, row.address_id).await;
        let logistics = fetch_order_logistics(&state, row.id).await?;
        let openid = fetch_openid(&state, row.user_id).await;
        list.push(build_order_resp(
            row, items, address, logistics, openid, false,
        ));
    }

    Ok(Json(ApiResponse::success(PagedOrders {
        list,
        total,
        page,
        page_size,
    })))
}

pub async fn get_order(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
) -> Result<Json<ApiResponse<OrderResp>>, AppError> {
    authorize_admin(&state, &headers, &[ORDER_LIST_VIEW]).await?;

    let order = sqlx::query_as::<_, OrderRow>(&format!("{} WHERE id = ?", ORDER_SELECT))
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("order not found".to_string()))?;

    let items = fetch_order_items(&state, order.id).await?;
    let address = fetch_address_snap(&state, order.address_id).await;
    let logistics = fetch_order_logistics(&state, order.id).await?;
    let openid = fetch_openid(&state, order.user_id).await;
    Ok(Json(ApiResponse::success(build_order_resp(
        &order, items, address, logistics, openid, false,
    ))))
}

pub async fn update_order_status(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<u64>,
    Json(body): Json<UpdateOrderStatusRequest>,
) -> Result<Json<ApiResponse<OrderResp>>, AppError> {
    authorize_admin(&state, &headers, &[ORDER_LIST_VIEW]).await?;

    if !(0..=4).contains(&body.status) {
        return Err(AppError::BadRequest(
            "invalid order status (expected 0-4)".to_string(),
        ));
    }

    let current_order = sqlx::query_as::<_, OrderRow>(&format!("{} WHERE id = ?", ORDER_SELECT))
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("order not found".to_string()))?;

    if current_order.status == 4 && body.status != 4 {
        return Err(AppError::BadRequest(
            "canceled orders cannot be reopened".to_string(),
        ));
    }

    let next_total_amount = if let Some(total_amount) = body.total_amount {
        if total_amount < 0 {
            return Err(AppError::BadRequest(
                "total amount cannot be negative".to_string(),
            ));
        }
        if current_order.status != 0 || body.status != 0 || current_order.paid_amount > 0 {
            return Err(AppError::BadRequest(
                "only unpaid orders can be repriced".to_string(),
            ));
        }
        total_amount
    } else {
        current_order.total_amount
    };

    let carrier = body.carrier.as_deref().unwrap_or("").trim();
    let tracking_no = body.tracking_no.as_deref().unwrap_or("").trim();
    let delivery_name = body.delivery_name.as_deref().unwrap_or("").trim();
    let delivery_phone = body.delivery_phone.as_deref().unwrap_or("").trim();
    let logistics_remark = body.logistics_remark.as_deref().map(str::trim);

    if body.status == 2 && tracking_no.is_empty() && delivery_phone.is_empty() {
        return Err(AppError::BadRequest(
            "修改为待收货时，需要填写物流单号或派件人手机号".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;
    let updated = sqlx::query(
        "UPDATE orders SET status = ?, total_amount = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND status = ?",
    )
    .bind(body.status)
    .bind(next_total_amount)
    .bind(id)
    .bind(current_order.status)
    .execute(&mut *tx)
    .await?;

    if updated.rows_affected() != 1 {
        tx.rollback().await?;
        return Err(AppError::BadRequest(
            "order status changed, please retry".to_string(),
        ));
    }

    if current_order.status == 0 && body.status == 4 {
        inventory::release_inventory_for_order(&mut tx, id).await?;
    }

    if body.status == 2 {
        sqlx::query(
            "INSERT INTO order_logistics (order_id, order_no, carrier, tracking_no, delivery_name, delivery_phone, remark) \
             VALUES (?, ?, ?, ?, ?, ?, ?) \
             ON DUPLICATE KEY UPDATE carrier = VALUES(carrier), tracking_no = VALUES(tracking_no), \
             delivery_name = VALUES(delivery_name), delivery_phone = VALUES(delivery_phone), remark = VALUES(remark)",
        )
        .bind(current_order.id)
        .bind(&current_order.order_no)
        .bind(carrier)
        .bind(tracking_no)
        .bind(delivery_name)
        .bind(delivery_phone)
        .bind(logistics_remark)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let order = sqlx::query_as::<_, OrderRow>(&format!("{} WHERE id = ?", ORDER_SELECT))
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    let items = fetch_order_items(&state, order.id).await?;
    let address = fetch_address_snap(&state, order.address_id).await;
    let logistics = fetch_order_logistics(&state, order.id).await?;
    let openid = fetch_openid(&state, order.user_id).await;
    Ok(Json(ApiResponse::success(build_order_resp(
        &order, items, address, logistics, openid, false,
    ))))
}
