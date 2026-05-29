use crate::error::AppError;
use crate::models::order::{build_order_resp, BalancePayResp, OrderResp, OrderRow};
use crate::routes::mini_app::order::{queries, PagedOrders, PayOrderResp};
use crate::state::AppState;

pub(super) async fn load_order_resp(
    state: &AppState,
    order: &OrderRow,
    openid: &str,
) -> Result<OrderResp, AppError> {
    let items = queries::fetch_order_items(state, order.id).await?;
    let address = queries::fetch_address_snap(state, order.address_id).await;
    let logistics = queries::fetch_order_logistics(state, order.id).await?;
    Ok(build_order_resp(
        order,
        items,
        address,
        logistics,
        openid.to_string(),
        true,
    ))
}

pub(super) async fn load_order_list_resp(
    state: &AppState,
    rows: &[OrderRow],
    openid: &str,
) -> Result<Vec<OrderResp>, AppError> {
    let mut list = Vec::with_capacity(rows.len());
    for row in rows {
        list.push(load_order_resp(state, row, openid).await?);
    }
    Ok(list)
}

pub(super) fn build_paged_orders(
    list: Vec<OrderResp>,
    total: i64,
    page: u64,
    page_size: u64,
) -> PagedOrders {
    PagedOrders {
        list,
        total,
        page,
        page_size,
    }
}

pub(super) fn build_pay_order_resp(
    success: bool,
    paid_amount: i64,
    order_status: Option<i64>,
    message: impl Into<String>,
) -> PayOrderResp {
    PayOrderResp {
        success,
        paid_amount,
        order_status,
        message: message.into(),
    }
}

pub(super) fn build_balance_pay_resp(
    success: bool,
    paid_amount: i64,
    balance_after: i64,
    order_status: Option<i64>,
    message: impl Into<String>,
) -> BalancePayResp {
    BalancePayResp {
        success,
        paid_amount,
        balance_after,
        order_status,
        message: message.into(),
    }
}
