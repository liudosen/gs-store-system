use anyhow::{Context, Result};
use sqlx::MySqlPool;

use crate::{
    common::{api::ApiError, errors::internal_error},
    domains::catalog::entity::ServiceItemRow,
};

pub async fn ensure_service_catalog_tables(db: &MySqlPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS service_items (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            code VARCHAR(64) NOT NULL UNIQUE,
            category_name VARCHAR(64) NOT NULL,
            name VARCHAR(64) NOT NULL,
            short_description VARCHAR(255) NOT NULL,
            badge VARCHAR(64) NOT NULL,
            base_price DECIMAL(10, 2) NOT NULL,
            duration_minutes INT NOT NULL,
            visible_in_customer TINYINT(1) NOT NULL DEFAULT 1,
            sort_order INT NOT NULL DEFAULT 0,
            is_active TINYINT(1) NOT NULL DEFAULT 1,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(db)
    .await
    .context("failed to ensure service_items table")?;

    ensure_service_item_column(
        db,
        "visible_in_customer",
        "ALTER TABLE service_items ADD COLUMN visible_in_customer TINYINT(1) NOT NULL DEFAULT 1",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS service_time_slots (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            slot_label VARCHAR(32) NOT NULL UNIQUE,
            sort_order INT NOT NULL DEFAULT 0,
            is_active TINYINT(1) NOT NULL DEFAULT 1
        )
        "#,
    )
    .execute(db)
    .await
    .context("failed to ensure service_time_slots table")?;

    seed_service_items(db).await?;
    seed_service_time_slots(db).await?;
    Ok(())
}

async fn ensure_service_item_column(
    db: &MySqlPool,
    column_name: &str,
    alter_sql: &str,
) -> Result<()> {
    let exists: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT 1
        FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE()
          AND TABLE_NAME = 'service_items'
          AND COLUMN_NAME = ?
        LIMIT 1
        "#,
    )
    .bind(column_name)
    .fetch_optional(db)
    .await
    .with_context(|| format!("failed to inspect service_items.{column_name}"))?;

    if exists.is_none() {
        sqlx::query(alter_sql)
            .execute(db)
            .await
            .with_context(|| format!("failed to ensure service_items.{column_name}"))?;
    }

    Ok(())
}

async fn seed_service_time_slots(db: &MySqlPool) -> Result<()> {
    let slots = [
        ("09:00-12:00", 1_i32),
        ("13:00-16:00", 2_i32),
        ("16:00-19:00", 3_i32),
    ];

    for (slot_label, sort_order) in slots {
        sqlx::query(
            r#"
            INSERT INTO service_time_slots (slot_label, sort_order, is_active)
            VALUES (?, ?, 1)
            ON DUPLICATE KEY UPDATE
                sort_order = VALUES(sort_order),
                is_active = 1
            "#,
        )
        .bind(slot_label)
        .bind(sort_order)
        .execute(db)
        .await
        .context("failed to seed service time slots")?;
    }

    Ok(())
}

async fn seed_service_items(db: &MySqlPool) -> Result<()> {
    let items = [
        (
            "escort-medical",
            "陪诊服务",
            "医院陪诊",
            "退役军人上门陪同就诊、排队取号、协助问诊。",
            "高频刚需",
            "168.00",
            180_i32,
            1_i32,
            1_i32,
        ),
        (
            "home-companion",
            "陪伴照护",
            "长者陪伴",
            "提供到家陪伴、聊天看护、散步陪同等基础陪护。",
            "暖心陪伴",
            "128.00",
            120_i32,
            1_i32,
            2_i32,
        ),
        (
            "home-cleaning",
            "生活服务",
            "轻家政整理",
            "针对长者家庭提供收纳整理、基础清洁与居家协助。",
            "便民到家",
            "158.00",
            150_i32,
            1_i32,
            3_i32,
        ),
        (
            "meal-delivery",
            "生活服务",
            "送餐到家",
            "按预约时段送餐上门，可附带代买代办。",
            "准时履约",
            "88.00",
            60_i32,
            1_i32,
            4_i32,
        ),
        (
            "community-support",
            "社区服务",
            "社区活动支持",
            "适用于社区活动组织、现场支持与陪同服务。",
            "活动支持",
            "138.00",
            180_i32,
            0_i32,
            5_i32,
        ),
    ];

    for (
        code,
        category_name,
        name,
        short_description,
        badge,
        base_price,
        duration_minutes,
        visible_in_customer,
        sort_order,
    ) in items
    {
        sqlx::query(
            r#"
            INSERT INTO service_items (
                code,
                category_name,
                name,
                short_description,
                badge,
                base_price,
                duration_minutes,
                visible_in_customer,
                sort_order,
                is_active
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
            ON DUPLICATE KEY UPDATE
                category_name = VALUES(category_name),
                name = VALUES(name),
                short_description = VALUES(short_description),
                badge = VALUES(badge),
                base_price = VALUES(base_price),
                duration_minutes = VALUES(duration_minutes),
                visible_in_customer = VALUES(visible_in_customer),
                sort_order = VALUES(sort_order),
                is_active = 1
            "#,
        )
        .bind(code)
        .bind(category_name)
        .bind(name)
        .bind(short_description)
        .bind(badge)
        .bind(base_price)
        .bind(duration_minutes)
        .bind(visible_in_customer)
        .bind(sort_order)
        .execute(db)
        .await
        .context("failed to seed service items")?;
    }

    Ok(())
}

pub async fn list_service_items(
    db: &MySqlPool,
    region_code: Option<&str>,
) -> Result<Vec<ServiceItemRow>, ApiError> {
    let _ = region_code;

    sqlx::query_as::<_, ServiceItemRow>(
        r#"
        SELECT
            id,
            code,
            category_name,
            name,
            short_description,
            badge,
            CAST(base_price AS CHAR) AS base_price,
            duration_minutes,
            visible_in_customer
        FROM service_items
        WHERE is_active = 1
        ORDER BY sort_order ASC, id ASC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(internal_error)
}

pub async fn find_service_item_by_id(
    db: &MySqlPool,
    service_item_id: i64,
) -> Result<Option<ServiceItemRow>, ApiError> {
    sqlx::query_as::<_, ServiceItemRow>(
        r#"
        SELECT
            id,
            code,
            category_name,
            name,
            short_description,
            badge,
            CAST(base_price AS CHAR) AS base_price,
            duration_minutes,
            visible_in_customer
        FROM service_items
        WHERE id = ? AND is_active = 1
        "#,
    )
    .bind(service_item_id)
    .fetch_optional(db)
    .await
    .map_err(internal_error)
}

pub async fn list_service_time_slots(db: &MySqlPool) -> Result<Vec<String>, ApiError> {
    sqlx::query_scalar(
        r#"
        SELECT slot_label
        FROM service_time_slots
        WHERE is_active = 1
        ORDER BY sort_order ASC, id ASC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(internal_error)
}
