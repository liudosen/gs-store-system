use axum::http::{HeaderMap, StatusCode};
use chrono::Local;
use uuid::Uuid;

use crate::{
    common::api::ApiError,
    domains::{
        auth::service as auth_service,
        catalog::repository as catalog_repository,
        customer::{repository as customer_repository, service as customer_service},
        order::{
            dto::{AcceptOrderData, AssignedOrderListData, AvailableOrderListData, CreateOrderData, CreateOrderRequest, CustomerOrderListQuery, OrderData, OrderDetailData, OrderListData},
            entity::ServiceOrderRow,
            repository,
        },
        veteran::repository as veteran_repository,
    },
    infra::state::AppState,
};

pub async fn create_order(
    state: &AppState,
    headers: &HeaderMap,
    payload: CreateOrderRequest,
) -> Result<CreateOrderData, ApiError> {
    let session = customer_service::require_customer_session(state, headers).await?;
    let service_item =
        catalog_repository::find_service_item_by_id(&state.db, payload.service_item_id)
            .await?
            .ok_or((StatusCode::BAD_REQUEST, "unknown service item"))?;
    let address =
        customer_repository::find_address_by_id(&state.db, session.user_id, payload.address_id)
            .await?
            .ok_or((StatusCode::BAD_REQUEST, "unknown address"))?;

    let service_date = payload.service_date.trim().to_string();
    let service_time_slot = payload.service_time_slot.trim().to_string();
    let note = payload.note.trim().to_string();
    if service_date.is_empty() || service_time_slot.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "service date and slot are required",
        ));
    }

    let service_address = format!(
        "{}{}{}",
        &address.city_name, &address.district_name, &address.detail_address
    );
    let dispatch_message = "订单已创建，等待服务者接单";

    let order = repository::create_order(
        &state.db,
        repository::CreateOrderParams {
            user_id: session.user_id,
            order_no: &generate_order_no(),
            service_item_id: service_item.id,
            service_item_name: &service_item.name,
            region_code: &address.region_code,
            region_name: &address.region_name,
            city_name: &address.city_name,
            district_name: &address.district_name,
            address_id: address.id,
            service_address: &service_address,
            contact_name: &address.contact_name,
            contact_phone: &address.contact_phone,
            service_date: &service_date,
            service_time_slot: &service_time_slot,
            note: &note,
            status: "matching",
            status_label: "待接单",
            dispatch_message,
        },
    )
    .await?;
    let order_data = map_order(order);

    if let Ok(msg) = serde_json::to_string(&serde_json::json!({
        "type": "new_order",
        "order": order_data
    })) {
        state.order_broadcaster.broadcast(&address.region_code, msg).await;
    }

    tracing::info!(
        order_id = order_data.id,
        order_no = %order_data.order_no,
        status = %order_data.status,
        region = %address.region_code,
        "order created"
    );

    Ok(CreateOrderData {
        order: order_data,
    })
}

pub async fn list_orders(
    state: &AppState,
    headers: &HeaderMap,
    query: CustomerOrderListQuery,
) -> Result<OrderListData, ApiError> {
    let session = customer_service::require_customer_session(state, headers).await?;
    let limit = query.limit.unwrap_or(10).clamp(1, 20);
    let offset = query.offset.unwrap_or(0).max(0);
    let category = query.category.unwrap_or_else(|| "active".to_string());

    let rows = repository::list_orders_by_user_id(
        &state.db,
        session.user_id,
        &category,
        offset,
        limit + 1,
    )
    .await?;

    let has_more = rows.len() as i64 > limit;
    let items = rows
        .into_iter()
        .take(limit as usize)
        .map(map_order)
        .collect::<Vec<_>>();

    Ok(OrderListData {
        next_offset: offset + items.len() as i64,
        has_more,
        items,
    })
}

pub async fn get_order_detail(
    state: &AppState,
    headers: &HeaderMap,
    order_id: i64,
) -> Result<OrderDetailData, ApiError> {
    let session = customer_service::require_customer_session(state, headers).await?;
    let order = repository::find_order_by_id(&state.db, session.user_id, order_id)
        .await?
        .ok_or((StatusCode::NOT_FOUND, "order not found"))?;

    Ok(OrderDetailData {
        order: map_order(order),
    })
}

fn generate_order_no() -> String {
    let uuid = Uuid::new_v4().simple().to_string();
    format!("XDJ{}{}", Local::now().format("%Y%m%d%H%M%S"), &uuid[..6])
}

pub async fn fetch_matching_orders(
    db: &sqlx::MySqlPool,
    region_code: &str,
) -> Result<Vec<OrderData>, ApiError> {
    let rows = repository::list_matching_orders_by_region(db, region_code).await?;
    tracing::info!(region_code, count = rows.len(), "fetched matching orders");
    Ok(rows.into_iter().map(map_order).collect())
}

pub fn map_order(order: ServiceOrderRow) -> OrderData {
    let status_label = order_status_label(&order.status).to_string();
    let status = order.status.clone();
    let assigned_veteran_name = order.assigned_veteran_name.clone();
    let dispatch_message = normalize_dispatch_message(
        &order.dispatch_message,
        &status,
        assigned_veteran_name.as_deref(),
    );
    let _ = (
        order.user_id,
        order.service_item_id,
        order.region_code,
        order.city_name,
        order.district_name,
        order.address_id,
        order.assigned_veteran_id,
    );

    OrderData {
        id: order.id,
        order_no: order.order_no,
        status,
        status_label,
        service_item_id: order.service_item_id,
        service_item_name: order.service_item_name,
        region_name: order.region_name,
        service_date: order.service_date,
        service_time_slot: order.service_time_slot,
        service_address: order.service_address,
        contact_name: order.contact_name,
        contact_phone: order.contact_phone,
        note: order.note,
        assigned_veteran_name,
        assigned_veteran_phone: order.assigned_veteran_phone,
        dispatch_message,
        created_at: order.created_at,
    }
}

fn order_status_label(status: &str) -> &'static str {
    match status {
        "matching" => "待接单",
        "assigned" => "已接单",
        "in_progress" => "服务中",
        "completed" => "已完成",
        "cancelled" => "已取消",
        _ => "未知状态",
    }
}

fn normalize_dispatch_message(
    message: &str,
    status: &str,
    assigned_veteran_name: Option<&str>,
) -> String {
    if message.contains("accepted order") {
        return format!(
            "退役军人 {} 已接单，等待上门服务开始",
            assigned_veteran_name.unwrap_or("服务者")
        );
    }

    if message.contains("auto dispatch") {
        return "订单已创建，等待服务者接单".to_string();
    }

    if message.is_empty() {
        return match status {
            "matching" => "订单已创建，等待服务者接单".to_string(),
            "assigned" => format!(
                "退役军人 {} 已接单，等待上门服务开始",
                assigned_veteran_name.unwrap_or("服务者")
            ),
            "in_progress" => "服务进行中".to_string(),
            "completed" => "服务已完成".to_string(),
            "cancelled" => "订单已取消".to_string(),
            _ => String::new(),
        };
    }

    message.to_string()
}

// ---- veteran 订单（service 层封装，避免 handler 跨域依赖） ----

pub async fn list_available_orders_for_veteran(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AvailableOrderListData, ApiError> {
    let session = auth_service::require_veteran_session(state, headers).await?;
    let profile = veteran_repository::find_veteran_by_id(&state.db, session.veteran_id)
        .await?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid session"))?;

    let items = fetch_matching_orders(&state.db, &profile.region_code).await?;
    Ok(AvailableOrderListData { items })
}

pub async fn accept_order_for_veteran(
    state: &AppState,
    headers: &HeaderMap,
    order_id: i64,
) -> Result<AcceptOrderData, ApiError> {
    let session = auth_service::require_veteran_session(state, headers).await?;
    let profile = veteran_repository::find_veteran_by_id(&state.db, session.veteran_id)
        .await?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid session"))?;

    let order = repository::veteran_accept_order_with_lock(
        &state.db,
        order_id,
        profile.id,
        &profile.name,
        &profile.phone,
    )
    .await?;

    tracing::info!(
        order_id,
        veteran_id = profile.id,
        veteran_name = %profile.name,
        "veteran accepted order"
    );

    // 接单后将服务者状态改为 busy
    if let Err(_e) = repository::mark_veteran_busy(&state.db, profile.id).await {
        tracing::warn!(veteran_id = profile.id, "failed to mark veteran busy after accept");
    }

    Ok(AcceptOrderData {
        order: map_order(order),
    })
}

pub async fn list_assigned_orders_for_veteran(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AssignedOrderListData, ApiError> {
    let session = auth_service::require_veteran_session(state, headers).await?;
    let rows = repository::list_assigned_orders_by_veteran(&state.db, session.veteran_id).await?;
    tracing::info!(veteran_id = session.veteran_id, count = rows.len(), "fetched assigned orders");
    Ok(AssignedOrderListData {
        items: rows.into_iter().map(map_order).collect(),
    })
}

pub async fn cancel_order_for_veteran(
    state: &AppState,
    headers: &HeaderMap,
    order_id: i64,
) -> Result<OrderDetailData, ApiError> {
    let session = auth_service::require_veteran_session(state, headers).await?;
    let profile = veteran_repository::find_veteran_by_id(&state.db, session.veteran_id)
        .await?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid session"))?;

    let order = repository::cancel_assigned_order(&state.db, order_id, profile.id).await?;

    // 取消后重新推送到同区域 WebSocket
    let order_data = map_order(order.clone());
    if let Ok(msg) = serde_json::to_string(&serde_json::json!({
        "type": "new_order",
        "order": order_data
    })) {
        state.order_broadcaster.broadcast(&profile.region_code, msg).await;
    }

    // 恢复服务者状态为 available
    if let Err(_e) = repository::mark_veteran_available(&state.db, profile.id).await {
        tracing::warn!(veteran_id = profile.id, "failed to mark veteran available after cancel");
    }

    tracing::info!(order_id, veteran_id = profile.id, "veteran cancelled order");

    Ok(OrderDetailData {
        order: order_data,
    })
}

pub async fn get_order_detail_for_veteran(
    state: &AppState,
    headers: &HeaderMap,
    order_id: i64,
) -> Result<OrderDetailData, ApiError> {
    let session = auth_service::require_veteran_session(state, headers).await?;
    let order = repository::find_assigned_order_by_veteran(&state.db, session.veteran_id, order_id)
        .await?
        .ok_or((StatusCode::NOT_FOUND, "order not found"))?;
    Ok(OrderDetailData {
        order: map_order(order),
    })
}
