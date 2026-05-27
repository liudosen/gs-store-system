use anyhow::{Context, Result};
use axum::{response::Html, routing::get, Router};
use chrono::{NaiveDate, Utc};
use dotenvy::dotenv;
use redis::aio::ConnectionManager;
use sqlx::mysql::MySqlPoolOptions;
use std::{path::Path, sync::Arc};
use tokio::time::{sleep, Duration};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::warn;
use tracing_appender::rolling::daily;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer, EnvFilter};

use crate::{
    domains::veteran::repository,
    domains::{catalog, customer, order},
    infra::{config::AppConfig, sms::SmsService, state::AppState},
    routes,
};

pub async fn bootstrap() -> Result<()> {
    let _ = dotenv();

    let _log_guard = init_tracing();

    let config = AppConfig::from_env()?;

    let db = MySqlPoolOptions::new()
        .max_connections(16)
        .connect(&config.database_url)
        .await
        .context("failed to connect mysql")?;
    repository::ensure_veteran_profile_table(&db).await?;
    customer::repository::ensure_customer_tables(&db).await?;
    catalog::repository::ensure_service_catalog_tables(&db).await?;
    order::repository::ensure_order_tables(&db).await?;
    repository::drop_legacy_users_table(&db).await?;

    let redis_client =
        redis::Client::open(config.redis_url.clone()).context("failed to create redis client")?;
    let redis = ConnectionManager::new(redis_client)
        .await
        .context("failed to connect redis")?;

    let sms_service = SmsService::from_config(&config.sms)
        .map(Arc::new)
        .map(Some)
        .unwrap_or_else(|error| {
            warn!("sms service disabled: {error:#}");
            None
        });

    let state = AppState::new(db, redis, sms_service);
    let app = build_app(state);

    // 启动日志清理定时任务：每天检查一次，删除 30 天前的旧日志
    tokio::spawn(async {
        loop {
            sleep(Duration::from_secs(3600 * 24)).await;
            clean_old_logs("logs", 30);
        }
    });

    let listener = tokio::net::TcpListener::bind(config.server_addr())
        .await
        .context("failed to bind server")?;

    println!("App running at http://{}", config.server_addr());
    axum::serve(listener, app)
        .await
        .context("server failed unexpectedly")?;

    Ok(())
}

fn init_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = daily("logs", "gs-store-system.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("gs_store_system=debug,tower_http=debug"));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_filter(env_filter),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_filter(EnvFilter::new("gs_store_system=info,tower_http=info")),
        )
        .init();

    guard
}

fn clean_old_logs(dir: &str, max_days: i64) {
    let cutoff = Utc::now().date_naive() - chrono::Duration::days(max_days);

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // 只处理 .log 文件，跳过非日志文件
        if path.extension().and_then(|ext| ext.to_str()) != Some("log") {
            continue;
        }

        let file_stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(stem) => stem,
            None => continue,
        };

        // tracing-appender daily rolling 格式: mcx.YYYY-MM-DD
        // file_stem 例如: mcx.2026-05-23
        let date_part = file_stem
            .strip_prefix("gs-store-system.")
            .unwrap_or(file_stem);

        let file_date = match NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => continue,
        };

        if file_date < cutoff {
            if let Err(e) = std::fs::remove_file(&path) {
                warn!("failed to delete old log file {}: {e}", path.display());
            } else {
                tracing::info!("deleted old log file: {}", path.display());
            }
        }
    }
}

fn build_app(state: AppState) -> Router {
    let frontend_dir = resolve_frontend_dir();

    Router::new()
        .nest("/api", routes::api_router())
        .route("/", get(spa_entry))
        .route("/veteran-join", get(spa_entry))
        .route("/veteran-join/", get(spa_entry))
        .route("/veteran-portal", get(veteran_portal_entry))
        .route("/veteran-portal/", get(veteran_portal_entry))
        .route("/xia-dao-jia/login", get(xia_dao_jia_entry))
        .route("/xiadaojia", get(xia_dao_jia_entry))
        .fallback_service(
            ServeDir::new(frontend_dir)
                .not_found_service(ServeFile::new(format!("{frontend_dir}/index.html"))),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn resolve_frontend_dir() -> &'static str {
    if Path::new("frontend/dist/index.html").exists() {
        "frontend/dist"
    } else {
        "frontend"
    }
}

async fn spa_entry() -> Result<Html<String>, axum::http::StatusCode> {
    render_spa_entry(None)
}

async fn veteran_portal_entry() -> Result<Html<String>, axum::http::StatusCode> {
    render_spa_entry(Some(RouteMetaOverride {
        title: "侠伴行服务者工作台",
        description: "侠伴行服务者工作台，支持订单处理、培训学习、个人资料维护与实时派单。",
        url: "https://www.mcx59481.cn/veteran-portal",
        image: "https://www.mcx59481.cn/share/veteran-portal-cover.jpg",
    }))
}

async fn xia_dao_jia_entry() -> Result<Html<String>, axum::http::StatusCode> {
    render_spa_entry(Some(RouteMetaOverride {
        title: "侠到家",
        description: "侠到家是迷彩侠面向社区家庭的高信任到家服务入口，支持服务预约、地址管理与订单追踪。",
        url: "https://www.mcx59481.cn/xiadaojia",
        image: "https://www.mcx59481.cn/share/xiadaojia-cover.jpg",
    }))
}

struct RouteMetaOverride<'a> {
    title: &'a str,
    description: &'a str,
    url: &'a str,
    image: &'a str,
}

fn render_spa_entry(meta: Option<RouteMetaOverride<'_>>) -> Result<Html<String>, axum::http::StatusCode> {
    let index_path = if Path::new("frontend/dist/index.html").exists() {
        Path::new("frontend/dist/index.html")
    } else {
        Path::new("frontend/index.html")
    };

    let html = std::fs::read_to_string(index_path).map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(meta) = meta else {
        return Ok(Html(html));
    };

    let meta_block = format!(
        r#"<!-- PAGE_META_START -->
    <meta property="og:type" content="website" />
    <meta property="og:site_name" content="迷彩侠实业有限公司" />
    <meta property="og:title" content="{title}" />
    <meta
      property="og:description"
      content="{description}"
    />
    <meta property="og:url" content="{url}" />
    <meta property="og:image" content="{image}" />
    <meta property="og:image:alt" content="迷彩侠退役军人服务平台" />
    <meta property="og:locale" content="zh_CN" />
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content="{title}" />
    <meta
      name="twitter:description"
      content="{description}"
    />
    <meta name="twitter:image" content="{image}" />
    <!-- PAGE_META_END -->"#,
        title = meta.title,
        description = meta.description,
        url = meta.url,
        image = meta.image,
    );

    let html = html
        .replace("<title>迷彩侠实业有限公司</title>", &format!("<title>{}</title>", meta.title))
        .replace(
            r#"<meta
      name="description"
      content="迷彩侠实业有限公司，聚焦退役军人创就业、实业合作与就业平台服务。"
    />"#,
            &format!(
                "<meta\n      name=\"description\"\n      content=\"{}\"\n    />",
                meta.description
            ),
        )
        .replace(
            r#"<!-- PAGE_META_START -->
    <meta property="og:type" content="website" />
    <meta property="og:site_name" content="迷彩侠实业有限公司" />
    <meta property="og:title" content="迷彩侠实业有限公司" />
    <meta
      property="og:description"
      content="迷彩侠实业有限公司，聚焦退役军人创就业、实业合作与就业平台服务。"
    />
    <meta property="og:url" content="https://www.mcx59481.cn/" />
    <meta property="og:image" content="https://www.mcx59481.cn/share/veteran-portal-cover.jpg" />
    <meta property="og:image:alt" content="迷彩侠退役军人服务平台" />
    <meta property="og:locale" content="zh_CN" />
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content="迷彩侠实业有限公司" />
    <meta
      name="twitter:description"
      content="迷彩侠实业有限公司，聚焦退役军人创就业、实业合作与就业平台服务。"
    />
    <meta name="twitter:image" content="https://www.mcx59481.cn/share/veteran-portal-cover.jpg" />
    <!-- PAGE_META_END -->"#,
            &meta_block,
        );

    Ok(Html(html))
}
