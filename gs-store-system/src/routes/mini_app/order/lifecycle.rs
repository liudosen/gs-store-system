use crate::error::AppError;
use crate::models::order::{CreateOrderRequest, OrderResp};
use crate::routes::mini_app::order::{guards, mapping, queries, ListOrdersQuery, PagedOrders};
use crate::routes::ApiResponse;
use crate::services::inventory::{self, InventoryChange};
use crate::state::AppState;
use axum::Json;
use chrono::Utc;

fn build_order_no(user_id: u64) -> String {
    format!(
        "{}{:04}",
        Utc::now().format("%Y%m%d%H%M%S%3f"),
        user_id % 10000
    )
}

pub(super) async fn confirm_my_order_received_impl(
    state: &AppState,
    openid: &str,
    id: u64,
) -> Result<Json<ApiResponse<OrderResp>>, AppError> {
    let user_id = queries::get_user_id_by_openid(state, openid).await?;
    let order = queries::fetch_owned_order(state, id, user_id).await?;

    if order.status != 2 {
        return Err(AppError::BadRequest(
            "only received orders can be confirmed".to_string(),
        ));
    }

    sqlx::query("UPDATE orders SET status = 3 WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;

    let updated = queries::fetch_order_row(state, id).await?;
    let resp = mapping::load_order_resp(state, &updated, openid).await?;
    Ok(Json(ApiResponse::success(resp)))
}

pub(super) async fn create_order_impl(
    state: &AppState,
    openid: &str,
    body: CreateOrderRequest,
) -> Result<Json<ApiResponse<OrderResp>>, AppError> {
    let user_id = queries::get_user_id_by_openid(state, openid).await?;

    if body.items.is_empty() {
        return Err(AppError::BadRequest(
            "create order requires at least one item".to_string(),
        ));
    }

    let address_id: u64 = body
        .address_id
        .parse()
        .map_err(|_| AppError::BadRequest("invalid addressId".to_string()))?;

    queries::ensure_address_owned_by_user(state, openid, address_id).await?;
    let dedup_key = guards::build_order_submit_guard_key(
        openid,
        &guards::build_order_request_fingerprint(&body),
    );
    if !guards::try_acquire_order_submit_guard(state, &dedup_key).await? {
        if let Some(existing) = guards::load_order_submit_guard(state, &dedup_key).await? {
            if let Ok(order_id) = existing.parse::<u64>() {
                let order = queries::fetch_owned_order(state, order_id, user_id).await?;
                let resp = mapping::load_order_resp(state, &order, openid).await?;
                return Ok(Json(ApiResponse::success(resp)));
            }
        }

        return Err(AppError::BadRequest(
            "duplicate order submission, please wait".to_string(),
        ));
    }

    let result = async {
        let order_no = build_order_no(user_id);
        let mut total_amount: i64 = 0;
        let mut resolved_items = Vec::with_capacity(body.items.len());
        for item_req in &body.items {
            let item = queries::resolve_order_item(state, item_req).await?;
            total_amount += item.subtotal;
            resolved_items.push(item);
        }

        let inventory_changes: Vec<InventoryChange> = resolved_items
            .iter()
            .map(|item| InventoryChange {
                sku_id: item.sku_id,
                spu_id: item.spu_id,
                quantity: item.quantity,
            })
            .collect();

        let mut tx = state.db.begin().await?;
        let inner_result = async {
            inventory::reserve_inventory(&mut tx, &inventory_changes).await?;
            let order_id = queries::insert_order_with_items(
                &mut tx,
                &order_no,
                user_id,
                address_id,
                body.remark.as_deref(),
                &resolved_items,
                total_amount,
            )
            .await?;
            Ok(order_id)
        }
        .await;

        match inner_result {
            Ok(order_id) => {
                tx.commit().await?;
                Ok(order_id)
            }
            Err(err) => {
                tx.rollback().await?;
                Err(err)
            }
        }
    }
    .await;

    match result {
        Ok(order_id) => {
            if let Err(err) =
                guards::set_order_submit_guard_order_id(state, &dedup_key, order_id).await
            {
                guards::clear_order_submit_guard(state, &dedup_key).await;
                return Err(err);
            }
            let order = queries::fetch_order_row(state, order_id).await?;
            let resp = mapping::load_order_resp(state, &order, openid).await?;
            Ok(Json(ApiResponse::success(resp)))
        }
        Err(err) => {
            guards::clear_order_submit_guard(state, &dedup_key).await;
            Err(err)
        }
    }
}

pub(super) async fn list_my_orders_impl(
    state: &AppState,
    openid: &str,
    q: ListOrdersQuery,
) -> Result<Json<ApiResponse<PagedOrders>>, AppError> {
    let user_id = queries::get_user_id_by_openid(state, openid).await?;

    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(10).min(50);
    let offset = (page - 1) * page_size;

    let (total, rows) =
        queries::fetch_order_page_rows(state, user_id, q.status, page_size, offset).await?;
    let list = mapping::load_order_list_resp(state, &rows, openid).await?;

    Ok(Json(ApiResponse::success(mapping::build_paged_orders(
        list, total, page, page_size,
    ))))
}

pub(super) async fn get_my_order_impl(
    state: &AppState,
    openid: &str,
    id: u64,
) -> Result<Json<ApiResponse<OrderResp>>, AppError> {
    let user_id = queries::get_user_id_by_openid(state, openid).await?;
    let order = queries::fetch_owned_order(state, id, user_id).await?;
    let resp = mapping::load_order_resp(state, &order, openid).await?;
    Ok(Json(ApiResponse::success(resp)))
}

pub(super) async fn cancel_my_order_impl(
    state: &AppState,
    openid: &str,
    id: u64,
) -> Result<Json<ApiResponse<OrderResp>>, AppError> {
    let user_id = queries::get_user_id_by_openid(state, openid).await?;
    let order = queries::fetch_owned_order(state, id, user_id).await?;

    if order.status != 0 {
        return Err(AppError::BadRequest(
            "only pending orders can be canceled".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;
    let updated = sqlx::query("UPDATE orders SET status = 4 WHERE id = ? AND status = 0")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    if updated.rows_affected() != 1 {
        tx.rollback().await?;
        return Err(AppError::BadRequest(
            "order status changed, please retry".to_string(),
        ));
    }

    inventory::release_inventory_for_order(&mut tx, id).await?;
    tx.commit().await?;

    let updated = queries::fetch_order_row(state, id).await?;
    let resp = mapping::load_order_resp(state, &updated, openid).await?;
    Ok(Json(ApiResponse::success(resp)))
}
