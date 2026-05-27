use anyhow::{Context, Result};
use sqlx::MySqlPool;

use crate::{
    common::{api::ApiError, errors::internal_error},
    domains::veteran::entity::{VeteranProfileDetailRow, VeteranProfileRow},
};

pub struct UpsertVeteranProfileParams<'a> {
    pub name: &'a str,
    pub id_number: &'a str,
    pub phone: &'a str,
    pub veteran_card_number: &'a str,
}

pub async fn ensure_veteran_profile_table(db: &MySqlPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS veteran_profiles (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            name VARCHAR(64) NOT NULL,
            id_number VARCHAR(32) NOT NULL UNIQUE,
            phone VARCHAR(32) NOT NULL UNIQUE,
            veteran_card_number VARCHAR(64) NOT NULL UNIQUE,
            region_code VARCHAR(32) NOT NULL DEFAULT 'sh-pudong',
            region_name VARCHAR(64) NOT NULL DEFAULT 'Pudong',
            service_tags VARCHAR(255) NOT NULL DEFAULT 'escort-medical,home-companion,home-cleaning,meal-delivery',
            is_dispatch_ready TINYINT(1) NOT NULL DEFAULT 1,
            service_status VARCHAR(32) NOT NULL DEFAULT 'available',
            rating_score DECIMAL(4,2) NOT NULL DEFAULT 4.80,
            completed_order_count INT NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(db)
    .await
    .context("failed to ensure veteran_profiles table")?;

    ensure_veteran_column(
        db,
        "region_code",
        "ALTER TABLE veteran_profiles ADD COLUMN region_code VARCHAR(32) NOT NULL DEFAULT 'sh-pudong'",
    )
    .await?;
    ensure_veteran_column(
        db,
        "region_name",
        "ALTER TABLE veteran_profiles ADD COLUMN region_name VARCHAR(64) NOT NULL DEFAULT 'Pudong'",
    )
    .await?;
    ensure_veteran_column(
        db,
        "service_tags",
        "ALTER TABLE veteran_profiles ADD COLUMN service_tags VARCHAR(255) NOT NULL DEFAULT 'escort-medical,home-companion,home-cleaning,meal-delivery'",
    )
    .await?;
    ensure_veteran_column(
        db,
        "is_dispatch_ready",
        "ALTER TABLE veteran_profiles ADD COLUMN is_dispatch_ready TINYINT(1) NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_veteran_column(
        db,
        "service_status",
        "ALTER TABLE veteran_profiles ADD COLUMN service_status VARCHAR(32) NOT NULL DEFAULT 'available'",
    )
    .await?;
    ensure_veteran_column(
        db,
        "rating_score",
        "ALTER TABLE veteran_profiles ADD COLUMN rating_score DECIMAL(4,2) NOT NULL DEFAULT 4.80",
    )
    .await?;
    ensure_veteran_column(
        db,
        "completed_order_count",
        "ALTER TABLE veteran_profiles ADD COLUMN completed_order_count INT NOT NULL DEFAULT 0",
    )
    .await?;

    seed_demo_veterans(db).await?;
    Ok(())
}

async fn ensure_veteran_column(db: &MySqlPool, column_name: &str, alter_sql: &str) -> Result<()> {
    let exists: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT 1
        FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE()
          AND TABLE_NAME = 'veteran_profiles'
          AND COLUMN_NAME = ?
        LIMIT 1
        "#,
    )
    .bind(column_name)
    .fetch_optional(db)
    .await
    .with_context(|| format!("failed to inspect veteran_profiles.{column_name}"))?;

    if exists.is_none() {
        sqlx::query(alter_sql)
            .execute(db)
            .await
            .with_context(|| format!("failed to ensure veteran_profiles.{column_name}"))?;
    }

    Ok(())
}

async fn seed_demo_veterans(db: &MySqlPool) -> Result<()> {
    let demo_veterans = [
        (
            "Zhang Lianguo",
            "demo-id-001",
            "13900000001",
            "vet-001",
            "sh-pudong",
            "Pudong",
            "escort-medical,home-companion",
        ),
        (
            "Wang Jianjun",
            "demo-id-002",
            "13900000002",
            "vet-002",
            "sh-minhang",
            "Minhang",
            "home-cleaning,meal-delivery,home-companion",
        ),
        (
            "Chen Zhiqiang",
            "demo-id-003",
            "13900000003",
            "vet-003",
            "sh-pudong",
            "Pudong",
            "escort-medical,meal-delivery",
        ),
    ];

    for (name, id_number, phone, veteran_card_number, region_code, region_name, service_tags) in
        demo_veterans
    {
        sqlx::query(
            r#"
            INSERT INTO veteran_profiles (
                name,
                id_number,
                phone,
                veteran_card_number,
                region_code,
                region_name,
                service_tags,
                is_dispatch_ready,
                service_status,
                rating_score,
                completed_order_count
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, 1, 'available', 4.90, 12)
            ON DUPLICATE KEY UPDATE
                name = VALUES(name),
                region_code = VALUES(region_code),
                region_name = VALUES(region_name),
                service_tags = VALUES(service_tags),
                is_dispatch_ready = 1
            "#,
        )
        .bind(name)
        .bind(id_number)
        .bind(phone)
        .bind(veteran_card_number)
        .bind(region_code)
        .bind(region_name)
        .bind(service_tags)
        .execute(db)
        .await
        .context("failed to seed veteran_profiles")?;
    }

    Ok(())
}

pub async fn drop_legacy_users_table(db: &MySqlPool) -> Result<()> {
    let _ = db;
    Ok(())
}

pub async fn find_veteran_profile_by_phone(
    db: &MySqlPool,
    phone: &str,
) -> Result<Option<VeteranProfileRow>, ApiError> {
    sqlx::query_as::<_, VeteranProfileRow>("SELECT id FROM veteran_profiles WHERE phone = ?")
        .bind(phone)
        .fetch_optional(db)
        .await
        .map_err(internal_error)
}

pub async fn find_veteran_profile_by_id(
    db: &MySqlPool,
    veteran_id: i64,
) -> Result<Option<VeteranProfileDetailRow>, ApiError> {
    sqlx::query_as::<_, VeteranProfileDetailRow>(
        r#"
        SELECT
            id,
            name,
            phone,
            veteran_card_number,
            region_code,
            region_name,
            service_tags,
            is_dispatch_ready,
            service_status,
            completed_order_count
        FROM veteran_profiles
        WHERE id = ?
        "#,
    )
    .bind(veteran_id)
    .fetch_optional(db)
    .await
    .map_err(internal_error)
}

pub async fn upsert_veteran_profile(
    db: &MySqlPool,
    params: UpsertVeteranProfileParams<'_>,
) -> Result<i64, ApiError> {
    if let Some(veteran) = find_veteran_profile_by_phone(db, params.phone).await? {
        sqlx::query(
            r#"
            UPDATE veteran_profiles
            SET
                name = ?,
                id_number = ?,
                veteran_card_number = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE phone = ?
            "#,
        )
        .bind(params.name)
        .bind(params.id_number)
        .bind(params.veteran_card_number)
        .bind(params.phone)
        .execute(db)
        .await
        .map_err(internal_error)?;

        return Ok(veteran.id);
    }

    let result = sqlx::query(
        r#"
        INSERT INTO veteran_profiles (name, id_number, phone, veteran_card_number)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(params.name)
    .bind(params.id_number)
    .bind(params.phone)
    .bind(params.veteran_card_number)
    .execute(db)
    .await
    .map_err(internal_error)?;

    Ok(result.last_insert_id() as i64)
}

pub async fn get_daily_stats(
    db: &MySqlPool,
    veteran_id: i64,
) -> Result<(i64, i64, i64, f64), ApiError> {
    let today_orders: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM service_orders WHERE assigned_veteran_id = ? AND service_date = CURDATE()",
    )
    .bind(veteran_id)
    .fetch_one(db)
    .await
    .map_err(internal_error)?;

    let today_completed: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM service_orders WHERE assigned_veteran_id = ? AND service_date = CURDATE() AND status = 'completed'",
    )
    .bind(veteran_id)
    .fetch_one(db)
    .await
    .map_err(internal_error)?;

    let month_orders: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM service_orders WHERE assigned_veteran_id = ? AND service_date LIKE CONCAT(DATE_FORMAT(CURDATE(), '%Y-%m'), '%')",
    )
    .bind(veteran_id)
    .fetch_one(db)
    .await
    .map_err(internal_error)?;

    let rating: (f64,) = sqlx::query_as(
        "SELECT CAST(COALESCE(rating_score, 0.0) AS DOUBLE) FROM veteran_profiles WHERE id = ?",
    )
    .bind(veteran_id)
    .fetch_one(db)
    .await
    .map_err(internal_error)?;

    Ok((today_orders.0, today_completed.0, month_orders.0, rating.0))
}

pub async fn update_veteran_region(
    db: &MySqlPool,
    veteran_id: i64,
    region_code: &str,
    region_name: &str,
) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        UPDATE veteran_profiles
        SET
            region_code = ?,
            region_name = ?,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(region_code)
    .bind(region_name)
    .bind(veteran_id)
    .execute(db)
    .await
    .map_err(internal_error)?;

    Ok(())
}

pub async fn find_veteran_by_id(
    db: &MySqlPool,
    veteran_id: i64,
) -> Result<Option<VeteranProfileDetailRow>, ApiError> {
    sqlx::query_as::<_, VeteranProfileDetailRow>(
        "SELECT id, name, phone, veteran_card_number, region_code, region_name, service_tags, is_dispatch_ready, service_status, completed_order_count FROM veteran_profiles WHERE id = ?",
    )
    .bind(veteran_id)
    .fetch_optional(db)
    .await
    .map_err(internal_error)
}
