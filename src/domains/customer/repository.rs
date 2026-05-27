use anyhow::{Context, Result};
use sqlx::MySqlPool;

use crate::{
    common::{api::ApiError, errors::internal_error},
    domains::customer::entity::{CustomerAddressRow, CustomerUserRow, ServiceRegionRow},
};

pub struct CreateOrUpdateCustomerUserParams<'a> {
    pub phone: &'a str,
}

pub struct UpdateRegionParams<'a> {
    pub user_id: i64,
    pub region_code: &'a str,
    pub region_name: &'a str,
}

pub struct UpsertAddressParams<'a> {
    pub address_id: Option<i64>,
    pub user_id: i64,
    pub region_code: &'a str,
    pub region_name: &'a str,
    pub city_name: &'a str,
    pub district_name: &'a str,
    pub detail_address: &'a str,
    pub contact_name: &'a str,
    pub contact_phone: &'a str,
    pub is_default: bool,
}

pub async fn ensure_customer_tables(db: &MySqlPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS customer_users (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            phone VARCHAR(32) NOT NULL UNIQUE,
            selected_region_code VARCHAR(32) NULL,
            selected_region_name VARCHAR(64) NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(db)
    .await
    .context("failed to ensure customer_users table")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS customer_addresses (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            user_id BIGINT NOT NULL,
            region_code VARCHAR(32) NOT NULL,
            region_name VARCHAR(64) NOT NULL,
            city_name VARCHAR(64) NOT NULL,
            district_name VARCHAR(64) NOT NULL,
            detail_address VARCHAR(255) NOT NULL,
            contact_name VARCHAR(64) NOT NULL,
            contact_phone VARCHAR(32) NOT NULL,
            is_default TINYINT(1) NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            INDEX idx_customer_addresses_user_id (user_id)
        )
        "#,
    )
    .execute(db)
    .await
    .context("failed to ensure customer_addresses table")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS service_regions (
            code VARCHAR(32) PRIMARY KEY,
            name VARCHAR(64) NOT NULL,
            city_name VARCHAR(64) NOT NULL,
            district_name VARCHAR(64) NOT NULL,
            sort_order INT NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(db)
    .await
    .context("failed to ensure service_regions table")?;

    seed_regions(db).await?;
    Ok(())
}

async fn seed_regions(db: &MySqlPool) -> Result<()> {
    let regions = [
        ("sh-pudong", "浦东新区", "上海", "浦东新区", 1_i32),
        ("sh-minhang", "闵行区", "上海", "闵行区", 2_i32),
        ("sh-baoshan", "宝山区", "上海", "宝山区", 3_i32),
        ("sh-songjiang", "松江区", "上海", "松江区", 4_i32),
    ];

    for (code, name, city_name, district_name, sort_order) in regions {
        sqlx::query(
            r#"
            INSERT INTO service_regions (code, name, city_name, district_name, sort_order)
            VALUES (?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                name = VALUES(name),
                city_name = VALUES(city_name),
                district_name = VALUES(district_name),
                sort_order = VALUES(sort_order)
            "#,
        )
        .bind(code)
        .bind(name)
        .bind(city_name)
        .bind(district_name)
        .bind(sort_order)
        .execute(db)
        .await
        .context("failed to seed service regions")?;
    }

    Ok(())
}

pub async fn find_region_by_code(
    db: &MySqlPool,
    region_code: &str,
) -> Result<Option<ServiceRegionRow>, ApiError> {
    sqlx::query_as::<_, ServiceRegionRow>(
        r#"
        SELECT code, name, city_name, district_name
        FROM service_regions
        WHERE code = ?
        "#,
    )
    .bind(region_code)
    .fetch_optional(db)
    .await
    .map_err(internal_error)
}

pub async fn list_regions(db: &MySqlPool) -> Result<Vec<ServiceRegionRow>, ApiError> {
    sqlx::query_as::<_, ServiceRegionRow>(
        r#"
        SELECT code, name, city_name, district_name
        FROM service_regions
        ORDER BY sort_order ASC, code ASC
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(internal_error)
}

pub async fn find_customer_user_by_phone(
    db: &MySqlPool,
    phone: &str,
) -> Result<Option<CustomerUserRow>, ApiError> {
    sqlx::query_as::<_, CustomerUserRow>(
        r#"
        SELECT id, phone, selected_region_code, selected_region_name
        FROM customer_users
        WHERE phone = ?
        "#,
    )
    .bind(phone)
    .fetch_optional(db)
    .await
    .map_err(internal_error)
}

pub async fn find_customer_user_by_id(
    db: &MySqlPool,
    user_id: i64,
) -> Result<Option<CustomerUserRow>, ApiError> {
    sqlx::query_as::<_, CustomerUserRow>(
        r#"
        SELECT id, phone, selected_region_code, selected_region_name
        FROM customer_users
        WHERE id = ?
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
    .map_err(internal_error)
}

pub async fn create_or_update_customer_user(
    db: &MySqlPool,
    params: CreateOrUpdateCustomerUserParams<'_>,
) -> Result<CustomerUserRow, ApiError> {
    if let Some(user) = find_customer_user_by_phone(db, params.phone).await? {
        return Ok(user);
    }

    let result = sqlx::query(
        r#"
        INSERT INTO customer_users (phone)
        VALUES (?)
        "#,
    )
    .bind(params.phone)
    .execute(db)
    .await
    .map_err(internal_error)?;

    let user_id = result.last_insert_id() as i64;
    find_customer_user_by_id(db, user_id)
        .await?
        .ok_or_else(|| internal_error("customer user created but not found"))
}

pub async fn update_customer_region(
    db: &MySqlPool,
    params: UpdateRegionParams<'_>,
) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        UPDATE customer_users
        SET
            selected_region_code = ?,
            selected_region_name = ?,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(params.region_code)
    .bind(params.region_name)
    .bind(params.user_id)
    .execute(db)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn list_addresses_by_user_id(
    db: &MySqlPool,
    user_id: i64,
) -> Result<Vec<CustomerAddressRow>, ApiError> {
    sqlx::query_as::<_, CustomerAddressRow>(
        r#"
        SELECT
            id,
            user_id,
            region_code,
            region_name,
            city_name,
            district_name,
            detail_address,
            contact_name,
            contact_phone,
            is_default
        FROM customer_addresses
        WHERE user_id = ?
        ORDER BY is_default DESC, id DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
    .map_err(internal_error)
}

pub async fn find_address_by_id(
    db: &MySqlPool,
    user_id: i64,
    address_id: i64,
) -> Result<Option<CustomerAddressRow>, ApiError> {
    sqlx::query_as::<_, CustomerAddressRow>(
        r#"
        SELECT
            id,
            user_id,
            region_code,
            region_name,
            city_name,
            district_name,
            detail_address,
            contact_name,
            contact_phone,
            is_default
        FROM customer_addresses
        WHERE user_id = ? AND id = ?
        "#,
    )
    .bind(user_id)
    .bind(address_id)
    .fetch_optional(db)
    .await
    .map_err(internal_error)
}

pub async fn clear_default_address(db: &MySqlPool, user_id: i64) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        UPDATE customer_addresses
        SET is_default = 0, updated_at = CURRENT_TIMESTAMP
        WHERE user_id = ?
        "#,
    )
    .bind(user_id)
    .execute(db)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn upsert_address(
    db: &MySqlPool,
    params: UpsertAddressParams<'_>,
) -> Result<CustomerAddressRow, ApiError> {
    if params.is_default {
        clear_default_address(db, params.user_id).await?;
    }

    let address_id = if let Some(address_id) = params.address_id {
        sqlx::query(
            r#"
            UPDATE customer_addresses
            SET
                region_code = ?,
                region_name = ?,
                city_name = ?,
                district_name = ?,
                detail_address = ?,
                contact_name = ?,
                contact_phone = ?,
                is_default = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(params.region_code)
        .bind(params.region_name)
        .bind(params.city_name)
        .bind(params.district_name)
        .bind(params.detail_address)
        .bind(params.contact_name)
        .bind(params.contact_phone)
        .bind(params.is_default)
        .bind(address_id)
        .bind(params.user_id)
        .execute(db)
        .await
        .map_err(internal_error)?;

        address_id
    } else {
        let result = sqlx::query(
            r#"
            INSERT INTO customer_addresses (
                user_id,
                region_code,
                region_name,
                city_name,
                district_name,
                detail_address,
                contact_name,
                contact_phone,
                is_default
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(params.user_id)
        .bind(params.region_code)
        .bind(params.region_name)
        .bind(params.city_name)
        .bind(params.district_name)
        .bind(params.detail_address)
        .bind(params.contact_name)
        .bind(params.contact_phone)
        .bind(params.is_default)
        .execute(db)
        .await
        .map_err(internal_error)?;

        result.last_insert_id() as i64
    };

    find_address_by_id(db, params.user_id, address_id)
        .await?
        .ok_or_else(|| internal_error("customer address saved but not found"))
}
