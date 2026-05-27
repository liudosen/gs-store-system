use anyhow::{Context, Result};
use sqlx::{MySqlPool, Row};

use crate::{
    common::{api::ApiError, errors::internal_error},
    domains::order::entity::ServiceOrderRow,
};

pub struct CreateOrderParams<'a> {
    pub user_id: i64,
    pub order_no: &'a str,
    pub service_item_id: i64,
    pub service_item_name: &'a str,
    pub region_code: &'a str,
    pub region_name: &'a str,
    pub city_name: &'a str,
    pub district_name: &'a str,
    pub address_id: i64,
    pub service_address: &'a str,
    pub contact_name: &'a str,
    pub contact_phone: &'a str,
    pub service_date: &'a str,
    pub service_time_slot: &'a str,
    pub note: &'a str,
    pub status: &'a str,
    pub status_label: &'a str,
    pub dispatch_message: &'a str,
}

pub async fn ensure_order_tables(db: &MySqlPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS service_orders (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            user_id BIGINT NOT NULL,
            order_no VARCHAR(64) NOT NULL UNIQUE,
            service_item_id BIGINT NOT NULL,
            service_item_name VARCHAR(64) NOT NULL,
            region_code VARCHAR(32) NOT NULL,
            region_name VARCHAR(64) NOT NULL,
            city_name VARCHAR(64) NOT NULL,
            district_name VARCHAR(64) NOT NULL,
            address_id BIGINT NOT NULL,
            service_address VARCHAR(255) NOT NULL,
            contact_name VARCHAR(64) NOT NULL,
            contact_phone VARCHAR(32) NOT NULL,
            service_date VARCHAR(32) NOT NULL,
            service_time_slot VARCHAR(32) NOT NULL,
            note VARCHAR(255) NOT NULL DEFAULT '',
            status VARCHAR(32) NOT NULL,
            status_label VARCHAR(64) NOT NULL,
            assigned_veteran_id BIGINT NULL,
            assigned_veteran_name VARCHAR(64) NULL,
            assigned_veteran_phone VARCHAR(32) NULL,
            dispatch_message VARCHAR(255) NOT NULL DEFAULT '',
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            INDEX idx_service_orders_user_id (user_id),
            INDEX idx_service_orders_region_code (region_code)
        )
        "#,
    )
    .execute(db)
    .await
    .context("failed to ensure service_orders table")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS order_status_logs (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            order_id BIGINT NOT NULL,
            status VARCHAR(32) NOT NULL,
            message VARCHAR(255) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_order_status_logs_order_id (order_id)
        )
        "#,
    )
    .execute(db)
    .await
    .context("failed to ensure order_status_logs table")?;

    Ok(())
}

pub async fn create_order(
    db: &MySqlPool,
    params: CreateOrderParams<'_>,
) -> Result<ServiceOrderRow, ApiError> {
    let result = sqlx::query(
        r#"
        INSERT INTO service_orders (
            user_id,
            order_no,
            service_item_id,
            service_item_name,
            region_code,
            region_name,
            city_name,
            district_name,
            address_id,
            service_address,
            contact_name,
            contact_phone,
            service_date,
            service_time_slot,
            note,
            status,
            status_label,
            dispatch_message
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(params.user_id)
    .bind(params.order_no)
    .bind(params.service_item_id)
    .bind(params.service_item_name)
    .bind(params.region_code)
    .bind(params.region_name)
    .bind(params.city_name)
    .bind(params.district_name)
    .bind(params.address_id)
    .bind(params.service_address)
    .bind(params.contact_name)
    .bind(params.contact_phone)
    .bind(params.service_date)
    .bind(params.service_time_slot)
    .bind(params.note)
    .bind(params.status)
    .bind(params.status_label)
    .bind(params.dispatch_message)
    .execute(db)
    .await
    .map_err(internal_error)?;

    let order_id = result.last_insert_id() as i64;
    insert_status_log(db, order_id, params.status, params.dispatch_message).await?;

    find_order_by_id(db, params.user_id, order_id)
        .await?
        .ok_or_else(|| internal_error("service order created but not found"))
}

pub async fn insert_status_log(
    db: &MySqlPool,
    order_id: i64,
    status: &str,
    message: &str,
) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        INSERT INTO order_status_logs (order_id, status, message)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(order_id)
    .bind(status)
    .bind(message)
    .execute(db)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn find_order_by_id(
    db: &MySqlPool,
    user_id: i64,
    order_id: i64,
) -> Result<Option<ServiceOrderRow>, ApiError> {
    sqlx::query_as::<_, ServiceOrderRow>(
        r#"
        SELECT
            id,
            user_id,
            order_no,
            service_item_id,
            service_item_name,
            region_code,
            region_name,
            city_name,
            district_name,
            address_id,
            service_address,
            contact_name,
            contact_phone,
            service_date,
            service_time_slot,
            note,
            status,
            status_label,
            assigned_veteran_id,
            assigned_veteran_name,
            assigned_veteran_phone,
            dispatch_message,
            DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') AS created_at
        FROM service_orders
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(order_id)
    .bind(user_id)
    .fetch_optional(db)
    .await
    .map_err(internal_error)
}

pub async fn list_orders_by_user_id(
    db: &MySqlPool,
    user_id: i64,
    category: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<ServiceOrderRow>, ApiError> {
    let status_clause = match category {
        "matching" => "status = 'matching'",
        "history" => "status = 'completed'",
        _ => "status IN ('assigned', 'in_progress')",
    };

    let sql = format!(
        r#"
        SELECT
            id,
            user_id,
            order_no,
            service_item_id,
            service_item_name,
            region_code,
            region_name,
            city_name,
            district_name,
            address_id,
            service_address,
            contact_name,
            contact_phone,
            service_date,
            service_time_slot,
            note,
            status,
            status_label,
            assigned_veteran_id,
            assigned_veteran_name,
            assigned_veteran_phone,
            dispatch_message,
            DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') AS created_at
        FROM service_orders
        WHERE user_id = ? AND {status_clause}
        ORDER BY id DESC
        LIMIT ? OFFSET ?
        "#
    );

    sqlx::query_as::<_, ServiceOrderRow>(&sql)
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await
        .map_err(internal_error)
}

pub async fn mark_veteran_busy(db: &MySqlPool, veteran_id: i64) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        UPDATE veteran_profiles
        SET service_status = 'busy', updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(veteran_id)
    .execute(db)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn mark_veteran_available(db: &MySqlPool, veteran_id: i64) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        UPDATE veteran_profiles
        SET service_status = 'available', updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(veteran_id)
    .execute(db)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn list_matching_orders_by_region(
    db: &MySqlPool,
    region_code: &str,
) -> Result<Vec<ServiceOrderRow>, ApiError> {
    sqlx::query_as::<_, ServiceOrderRow>(
        r#"
        SELECT
            id, user_id, order_no, service_item_id, service_item_name,
            region_code, region_name, city_name, district_name,
            address_id, service_address, contact_name, contact_phone,
            service_date, service_time_slot, note,
            status, status_label,
            assigned_veteran_id, assigned_veteran_name, assigned_veteran_phone,
            dispatch_message,
            DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') AS created_at
        FROM service_orders
        WHERE status = 'matching' AND region_code = ?
        ORDER BY id ASC
        "#,
    )
    .bind(region_code)
    .fetch_all(db)
    .await
    .map_err(internal_error)
}

pub async fn veteran_accept_order_with_lock(
    db: &MySqlPool,
    order_id: i64,
    veteran_id: i64,
    veteran_name: &str,
    veteran_phone: &str,
) -> Result<ServiceOrderRow, ApiError> {
    let dispatch_message = format!("退役军人 {} 已接单，等待上门服务开始", veteran_name);
    let mut tx = db.begin().await.map_err(internal_error)?;

    let veteran_exists = sqlx::query("SELECT id FROM veteran_profiles WHERE id = ? FOR UPDATE")
        .bind(veteran_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(internal_error)?;

    if veteran_exists.is_none() {
        return Err((axum::http::StatusCode::NOT_FOUND, "veteran not found"));
    }

    let order_row = sqlx::query(
        r#"
        SELECT user_id, status, service_date, service_time_slot
        FROM service_orders
        WHERE id = ?
        FOR UPDATE
        "#,
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?
    .ok_or((axum::http::StatusCode::NOT_FOUND, "order not found"))?;

    let user_id: i64 = order_row.get("user_id");
    let order_status: String = order_row.get("status");
    let service_date: String = order_row.get("service_date");
    let service_time_slot: String = order_row.get("service_time_slot");
    if order_status != "matching" {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "order already taken or not found",
        ));
    }

    let normalized_slot = normalize_service_time_slot(&service_time_slot);

    let conflict_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(1)
        FROM service_orders
        WHERE assigned_veteran_id = ?
          AND status IN ('assigned', 'in_progress')
          AND service_date = ?
          AND REPLACE(REPLACE(REPLACE(service_time_slot, '上午 ', ''), '下午 ', ''), '中午 ', '') = ?
        FOR UPDATE
        "#,
    )
    .bind(veteran_id)
    .bind(&service_date)
    .bind(&normalized_slot)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_error)?;

    if conflict_count > 0 {
        return Err((
            axum::http::StatusCode::CONFLICT,
            "该时间段已有服务安排，不能接单",
        ));
    }

    sqlx::query(
        r#"
        UPDATE service_orders
        SET
            status = 'assigned',
            status_label = '已接单',
            assigned_veteran_id = ?,
            assigned_veteran_name = ?,
            assigned_veteran_phone = ?,
            dispatch_message = ?,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(veteran_id)
    .bind(veteran_name)
    .bind(veteran_phone)
    .bind(&dispatch_message)
    .bind(order_id)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        r#"
        INSERT INTO order_status_logs (order_id, status, message)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(order_id)
    .bind("assigned")
    .bind(&dispatch_message)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;

    find_order_by_id(db, user_id, order_id)
        .await?
        .ok_or_else(|| internal_error("accepted order not found"))
}

fn normalize_service_time_slot(slot: &str) -> String {
    slot.replace("上午 ", "")
        .replace("下午 ", "")
        .replace("中午 ", "")
        .trim()
        .to_string()
}

pub async fn list_assigned_orders_by_veteran(
    db: &MySqlPool,
    veteran_id: i64,
) -> Result<Vec<ServiceOrderRow>, ApiError> {
    sqlx::query_as::<_, ServiceOrderRow>(
        r#"
        SELECT
            id, user_id, order_no, service_item_id, service_item_name,
            region_code, region_name, city_name, district_name,
            address_id, service_address, contact_name, contact_phone,
            service_date, service_time_slot, note,
            status, status_label,
            assigned_veteran_id, assigned_veteran_name, assigned_veteran_phone,
            dispatch_message,
            DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') AS created_at
        FROM service_orders
        WHERE assigned_veteran_id = ? AND status IN ('assigned', 'in_progress', 'completed')
        ORDER BY id DESC
        "#,
    )
    .bind(veteran_id)
    .fetch_all(db)
    .await
    .map_err(internal_error)
}

pub async fn cancel_assigned_order(
    db: &MySqlPool,
    order_id: i64,
    veteran_id: i64,
) -> Result<ServiceOrderRow, ApiError> {
    let affected = sqlx::query(
        r#"
        UPDATE service_orders
        SET status = 'matching',
            status_label = '待接单',
            assigned_veteran_id = NULL,
            assigned_veteran_name = NULL,
            assigned_veteran_phone = NULL,
            dispatch_message = '订单已释放，等待其他服务者接单',
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND assigned_veteran_id = ? AND status IN ('assigned', 'in_progress')
        "#,
    )
    .bind(order_id)
    .bind(veteran_id)
    .execute(db)
    .await
    .map_err(internal_error)?
    .rows_affected();

    if affected == 0 {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "订单不存在或状态不允许取消",
        ));
    }

    insert_status_log(db, order_id, "matching", "服务者取消订单，订单已释放").await?;

    let (user_id,): (i64,) = sqlx::query_as("SELECT user_id FROM service_orders WHERE id = ?")
        .bind(order_id)
        .fetch_optional(db)
        .await
        .map_err(internal_error)?
        .ok_or((axum::http::StatusCode::NOT_FOUND, "order not found"))?;

    find_order_by_id(db, user_id, order_id)
        .await?
        .ok_or_else(|| internal_error("cancelled order not found"))
}

pub async fn find_assigned_order_by_veteran(
    db: &MySqlPool,
    veteran_id: i64,
    order_id: i64,
) -> Result<Option<ServiceOrderRow>, ApiError> {
    sqlx::query_as::<_, ServiceOrderRow>(
        r#"
        SELECT
            id, user_id, order_no, service_item_id, service_item_name,
            region_code, region_name, city_name, district_name,
            address_id, service_address, contact_name, contact_phone,
            service_date, service_time_slot, note,
            status, status_label,
            assigned_veteran_id, assigned_veteran_name, assigned_veteran_phone,
            dispatch_message,
            DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s') AS created_at
        FROM service_orders
        WHERE id = ? AND assigned_veteran_id = ?
        "#,
    )
    .bind(order_id)
    .bind(veteran_id)
    .fetch_optional(db)
    .await
    .map_err(internal_error)
}
