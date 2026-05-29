use crate::error::AppError;
use crate::models::order::{
    CreateOrderItemReq, OrderAddressSnap, OrderItemRow, OrderLogisticsRow, OrderRow,
};
use crate::state::AppState;
use sqlx::Executor;

const ORDER_SELECT_SQL: &str = "SELECT id, order_no, external_order_no, user_id, address_id, status, total_amount, paid_amount, discount_amount, remark, created_at, updated_at FROM orders WHERE id = ?";
const ORDER_ITEM_SELECT_SQL: &str = "SELECT oi.id, oi.order_id, oi.order_no, oi.spu_id, oi.sku_id, COALESCE(gs.is_default, 0) AS is_default_sku, oi.goods_title, oi.goods_image, oi.spec_info, oi.unit_price, oi.quantity, oi.subtotal FROM order_items oi LEFT JOIN goods_skus gs ON gs.id = oi.sku_id WHERE oi.order_id = ? ORDER BY oi.id";
const ORDER_LOGISTICS_SELECT_SQL: &str = "SELECT id, order_id, order_no, carrier, tracking_no, delivery_name, delivery_phone, remark, created_at, updated_at FROM order_logistics WHERE order_id = ?";
const ADDRESS_SELECT_SQL: &str = "SELECT id, receiver_name, phone, province, city, district, detail_address, label FROM addresses WHERE id = ?";
const WECHAT_USER_ID_SQL: &str = "SELECT id FROM wechat_users WHERE openid = ?";
const WECHAT_ID_CARD_SQL: &str =
    "SELECT COALESCE(id_card_number, '') AS id_card_number FROM wechat_users WHERE openid = ?";
const ADDRESS_OWNER_SQL: &str = "SELECT open_id FROM addresses WHERE id = ?";
const ORDER_INSERT_SQL: &str = "INSERT INTO orders (order_no, user_id, address_id, status, total_amount, paid_amount, discount_amount, remark) VALUES (?, ?, ?, 0, ?, 0, 0, ?)";
const ORDER_ITEM_INSERT_SQL: &str = "INSERT INTO order_items (order_id, order_no, spu_id, sku_id, goods_title, goods_image, spec_info, unit_price, quantity, subtotal) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

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

#[derive(sqlx::FromRow)]
struct SkuLookup {
    id: u64,
    spu_id: u64,
    sale_price: i64,
    spec_info: String,
    title: String,
    primary_image: String,
}

#[derive(sqlx::FromRow)]
struct UserIdCardRow {
    id_card_number: String,
}

pub(super) struct ResolvedOrderItem {
    pub(super) sku_id: u64,
    pub(super) spu_id: u64,
    pub(super) goods_title: String,
    pub(super) goods_image: String,
    pub(super) spec_info: String,
    pub(super) unit_price: i64,
    pub(super) quantity: i32,
    pub(super) subtotal: i64,
}

pub(super) async fn fetch_current_balance_on<'e, E>(
    executor: E,
    openid: &str,
) -> Result<i64, AppError>
where
    E: Executor<'e, Database = sqlx::MySql>,
{
    let balance: i64 = sqlx::query_scalar(
        "SELECT COALESCE((SELECT balance FROM balance_accounts WHERE openid = ?), (SELECT balance_after FROM balance_transactions WHERE openid = ? ORDER BY id DESC LIMIT 1), 0)",
    )
    .bind(openid)
    .bind(openid)
    .fetch_one(executor)
    .await?;

    Ok(balance)
}

pub(super) async fn fetch_order_row(state: &AppState, order_id: u64) -> Result<OrderRow, AppError> {
    sqlx::query_as::<_, OrderRow>(ORDER_SELECT_SQL)
        .bind(order_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("order not found".to_string()))
}

pub(super) async fn fetch_owned_order(
    state: &AppState,
    order_id: u64,
    user_id: u64,
) -> Result<OrderRow, AppError> {
    let order = fetch_order_row(state, order_id).await?;
    if order.user_id != user_id {
        return Err(AppError::PermissionDenied);
    }
    Ok(order)
}

pub(super) async fn fetch_order_items(
    state: &AppState,
    order_id: u64,
) -> Result<Vec<OrderItemRow>, AppError> {
    let items = sqlx::query_as::<_, OrderItemRow>(ORDER_ITEM_SELECT_SQL)
        .bind(order_id)
        .fetch_all(&state.db)
        .await?;
    Ok(items)
}

pub(super) async fn fetch_order_logistics(
    state: &AppState,
    order_id: u64,
) -> Result<Option<OrderLogisticsRow>, AppError> {
    let logistics = sqlx::query_as::<_, OrderLogisticsRow>(ORDER_LOGISTICS_SELECT_SQL)
        .bind(order_id)
        .fetch_optional(&state.db)
        .await?;
    Ok(logistics)
}

pub(super) async fn fetch_address_snap(
    state: &AppState,
    address_id: Option<u64>,
) -> Option<OrderAddressSnap> {
    let id = address_id?;
    sqlx::query_as::<_, AddressRow>(ADDRESS_SELECT_SQL)
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|row| OrderAddressSnap {
            id: row.id.to_string(),
            receiver_name: row.receiver_name,
            phone: row.phone,
            province: row.province,
            city: row.city,
            district: row.district,
            detail_address: row.detail_address,
            label: row.label,
        })
}

pub(super) async fn get_user_id_by_openid(state: &AppState, openid: &str) -> Result<u64, AppError> {
    let user_id: u64 = sqlx::query_scalar(WECHAT_USER_ID_SQL)
        .bind(openid)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("user not found".to_string()))?;
    Ok(user_id)
}

pub(super) async fn fetch_user_id_card_number(
    state: &AppState,
    openid: &str,
) -> Result<String, AppError> {
    let row = sqlx::query_as::<_, UserIdCardRow>(WECHAT_ID_CARD_SQL)
        .bind(openid)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("user not found".to_string()))?;
    Ok(row.id_card_number)
}

pub(super) async fn ensure_address_owned_by_user(
    state: &AppState,
    openid: &str,
    address_id: u64,
) -> Result<(), AppError> {
    let owner: Option<String> = sqlx::query_scalar(ADDRESS_OWNER_SQL)
        .bind(address_id)
        .fetch_optional(&state.db)
        .await?;

    match owner {
        None => Err(AppError::NotFound("address not found".to_string())),
        Some(owner_openid) if owner_openid != openid => Err(AppError::PermissionDenied),
        _ => Ok(()),
    }
}

pub(super) async fn resolve_order_item(
    state: &AppState,
    item_req: &CreateOrderItemReq,
) -> Result<ResolvedOrderItem, AppError> {
    if item_req.quantity <= 0 {
        return Err(AppError::BadRequest(
            "quantity must be positive".to_string(),
        ));
    }

    let sku_row: SkuLookup = if let Some(ref sku_id_str) = item_req.sku_id {
        let sku_id: u64 = sku_id_str
            .parse()
            .map_err(|_| AppError::BadRequest("invalid skuId".to_string()))?;
        sqlx::query_as::<_, SkuLookup>(
            "SELECT gs.id, gs.spu_id, gs.sale_price, gs.spec_info, g.title, g.primary_image FROM goods_skus gs JOIN goods g ON g.id = gs.spu_id WHERE gs.id = ? AND g.status = 1",
        )
        .bind(sku_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound(format!(
            "SKU {} not found or product offline",
            sku_id
        )))?
    } else if let Some(ref spu_id_str) = item_req.spu_id {
        let spu_id: u64 = spu_id_str
            .parse()
            .map_err(|_| AppError::BadRequest("invalid spuId".to_string()))?;
        sqlx::query_as::<_, SkuLookup>(
            "SELECT gs.id, gs.spu_id, gs.sale_price, gs.spec_info, g.title, g.primary_image FROM goods_skus gs JOIN goods g ON g.id = gs.spu_id WHERE gs.spu_id = ? AND g.status = 1 ORDER BY gs.is_default ASC, gs.id LIMIT 1",
        )
        .bind(spu_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound(format!(
            "SPU {} has no available SKU or product is offline",
            spu_id
        )))?
    } else {
        return Err(AppError::BadRequest(
            "each item must provide skuId or spuId".to_string(),
        ));
    };

    let subtotal = sku_row.sale_price * item_req.quantity as i64;
    Ok(ResolvedOrderItem {
        sku_id: sku_row.id,
        spu_id: sku_row.spu_id,
        goods_title: sku_row.title,
        goods_image: sku_row.primary_image,
        spec_info: sku_row.spec_info,
        unit_price: sku_row.sale_price,
        quantity: item_req.quantity,
        subtotal,
    })
}

pub(super) async fn insert_order_with_items(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    order_no: &str,
    user_id: u64,
    address_id: u64,
    remark: Option<&str>,
    items: &[ResolvedOrderItem],
    total_amount: i64,
) -> Result<u64, AppError> {
    let order_insert = sqlx::query(ORDER_INSERT_SQL)
        .bind(order_no)
        .bind(user_id)
        .bind(address_id)
        .bind(total_amount)
        .bind(remark)
        .execute(&mut **tx)
        .await?;

    let order_id = order_insert.last_insert_id();

    for item in items {
        sqlx::query(ORDER_ITEM_INSERT_SQL)
            .bind(order_id)
            .bind(order_no)
            .bind(item.spu_id)
            .bind(item.sku_id)
            .bind(&item.goods_title)
            .bind(&item.goods_image)
            .bind(&item.spec_info)
            .bind(item.unit_price)
            .bind(item.quantity)
            .bind(item.subtotal)
            .execute(&mut **tx)
            .await?;
    }

    Ok(order_id)
}

pub(super) async fn fetch_order_page_rows(
    state: &AppState,
    user_id: u64,
    status: Option<i8>,
    page_size: u64,
    offset: u64,
) -> Result<(i64, Vec<OrderRow>), AppError> {
    let (count_sql, list_sql) = if status.is_some() {
        (
            "SELECT COUNT(*) FROM orders WHERE user_id = ? AND status = ?".to_string(),
            "SELECT id, order_no, external_order_no, user_id, address_id, status, total_amount, paid_amount, discount_amount, remark, created_at, updated_at FROM orders WHERE user_id = ? AND status = ? ORDER BY id DESC LIMIT ? OFFSET ?".to_string(),
        )
    } else {
        (
            "SELECT COUNT(*) FROM orders WHERE user_id = ?".to_string(),
            "SELECT id, order_no, external_order_no, user_id, address_id, status, total_amount, paid_amount, discount_amount, remark, created_at, updated_at FROM orders WHERE user_id = ? ORDER BY id DESC LIMIT ? OFFSET ?".to_string(),
        )
    };

    let total: i64 = if let Some(st) = status {
        sqlx::query_scalar(&count_sql)
            .bind(user_id)
            .bind(st)
            .fetch_one(&state.db)
            .await?
    } else {
        sqlx::query_scalar(&count_sql)
            .bind(user_id)
            .fetch_one(&state.db)
            .await?
    };

    let rows: Vec<OrderRow> = if let Some(st) = status {
        sqlx::query_as::<_, OrderRow>(&list_sql)
            .bind(user_id)
            .bind(st)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query_as::<_, OrderRow>(&list_sql)
            .bind(user_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&state.db)
            .await?
    };

    Ok((total, rows))
}
