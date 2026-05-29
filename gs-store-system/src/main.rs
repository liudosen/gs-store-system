mod config;
mod error;
mod logging;
mod models;
mod routes;
mod services;
mod state;

use state::AppState;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::from_filename_override(".env");

    logging::init_logging(None);

    tracing::info!("Starting GS Store System API");

    let config = config::Config::from_env()?;
    tracing::info!(
        "Configuration loaded: {}:{}",
        config.server_host,
        config.server_port
    );
    tracing::info!(
        max_connections = config.database_max_connections,
        min_connections = config.database_min_connections,
        acquire_timeout_seconds = config.database_acquire_timeout_seconds,
        idle_timeout_seconds = config.database_idle_timeout_seconds,
        max_lifetime_seconds = config.database_max_lifetime_seconds,
        "Database pool configuration loaded"
    );

    let db = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(config.database_max_connections)
        .min_connections(config.database_min_connections)
        .acquire_timeout(Duration::from_secs(config.database_acquire_timeout_seconds))
        .idle_timeout(Duration::from_secs(config.database_idle_timeout_seconds))
        .max_lifetime(Duration::from_secs(config.database_max_lifetime_seconds))
        .test_before_acquire(true)
        .connect(&config.database_url)
        .await?;
    tracing::info!("Database connected");

    let redis_cfg = deadpool_redis::Config::from_url(config.redis_url.as_str());
    let redis_pool = redis_cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    {
        let mut redis_conn = redis_pool.get().await?;
        redis::cmd("PING")
            .query_async::<_, String>(&mut redis_conn)
            .await?;
    }
    tracing::info!("Redis pool connected");

    let skip_migrations = std::env::var("SKIP_MIGRATIONS")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
        .unwrap_or(false);

    if skip_migrations {
        tracing::warn!("Skipping database migrations because SKIP_MIGRATIONS is enabled");
    } else {
        sqlx::migrate!("./migrations").run(&db).await?;
        tracing::info!("Migrations completed");
    }

    if let (Ok(admin_username), Ok(admin_password)) = (
        std::env::var("ADMIN_USERNAME"),
        std::env::var("ADMIN_PASSWORD"),
    ) {
        let exists: Option<(u64,)> =
            sqlx::query_as("SELECT id FROM admin_users WHERE username = ?")
                .bind(&admin_username)
                .fetch_optional(&db)
                .await?;

        if exists.is_none() {
            let password_hash = bcrypt::hash(&admin_password, config.bcrypt_cost)?;
            sqlx::query(
                "INSERT INTO admin_users (username, password_hash, role, permission_codes, is_active) VALUES (?, ?, 'admin', '[]', 1)",
            )
            .bind(&admin_username)
            .bind(&password_hash)
            .execute(&db)
            .await?;
            tracing::info!("Initial admin user created: {}", admin_username);
        } else {
            tracing::info!("Admin user already exists, skipping creation");
        }
    }

    let app_state = Arc::new(AppState {
        db,
        redis: redis_pool,
        jwt_secret: config.jwt_secret.clone(),
        jwt_expiry_hours: config.jwt_expiry_hours,
        auth_require_redis_session: config.auth_require_redis_session,
        bcrypt_cost: config.bcrypt_cost,
        wechat_appid: config.wechat_appid.clone(),
        wechat_secret: config.wechat_secret.clone(),
        dev_wechat_openid: config.dev_wechat_openid.clone(),
        jk_seller_username: config.jk_seller_username.clone(),
        jk_seller_password: config.jk_seller_password.clone(),
        oss_endpoint: config.oss_endpoint.clone(),
        oss_access_key_id: config.oss_access_key_id.clone(),
        oss_access_key_secret: config.oss_access_key_secret.clone(),
        oss_bucket: config.oss_bucket.clone(),
        oss_domain: config.oss_domain.clone(),
    });

    let mut warmup_redis = app_state.redis_conn().await?;
    if let Err(e) = services::jk_pay::warmup(
        &mut warmup_redis,
        &app_state.jk_seller_username,
        &app_state.jk_seller_password,
    )
    .await
    {
        tracing::warn!("JK payment warmup failed, continuing startup: {}", e);
    }

    let app = routes::app::build_app(app_state);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("API service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
