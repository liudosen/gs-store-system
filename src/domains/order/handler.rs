use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};

use crate::{
    common::api::{ApiResponse, ApiResult},
    domains::order::{
        dto::{AcceptOrderData, AssignedOrderListData, AvailableOrderListData, CreateOrderData, CreateOrderRequest, CustomerOrderListQuery, OrderDetailData, OrderListData},
        service as order_service,
    },
    infra::state::AppState,
};

pub async fn create_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateOrderRequest>,
) -> ApiResult<CreateOrderData> {
    let data = order_service::create_order(&state, &headers, payload).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "订单已创建".to_string(),
        data,
    }))
}

pub async fn list_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CustomerOrderListQuery>,
) -> ApiResult<OrderListData> {
    let data = order_service::list_orders(&state, &headers, query).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "获取成功".to_string(),
        data,
    }))
}

pub async fn get_order_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<i64>,
) -> ApiResult<OrderDetailData> {
    let data = order_service::get_order_detail(&state, &headers, order_id).await?;

    Ok(Json(ApiResponse {
        success: true,
        message: "获取成功".to_string(),
        data,
    }))
}

// ---- veteran 订单接口 ----

pub async fn list_available_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<AvailableOrderListData> {
    let data = order_service::list_available_orders_for_veteran(&state, &headers).await?;
    Ok(Json(ApiResponse { success: true, message: "ok".to_string(), data }))
}

pub async fn accept_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<i64>,
) -> ApiResult<AcceptOrderData> {
    let data = order_service::accept_order_for_veteran(&state, &headers, order_id).await?;
    Ok(Json(ApiResponse { success: true, message: "接单成功".to_string(), data }))
}

pub async fn list_assigned_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<AssignedOrderListData> {
    let data = order_service::list_assigned_orders_for_veteran(&state, &headers).await?;
    Ok(Json(ApiResponse { success: true, message: "ok".to_string(), data }))
}

pub async fn cancel_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<i64>,
) -> ApiResult<OrderDetailData> {
    let data = order_service::cancel_order_for_veteran(&state, &headers, order_id).await?;
    Ok(Json(ApiResponse { success: true, message: "订单已取消".to_string(), data }))
}

pub async fn get_assigned_order_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<i64>,
) -> ApiResult<OrderDetailData> {
    let data = order_service::get_order_detail_for_veteran(&state, &headers, order_id).await?;
    Ok(Json(ApiResponse { success: true, message: "ok".to_string(), data }))
}
