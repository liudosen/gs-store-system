mod captcha;
mod client;
mod crypto;
mod payment_flow;
mod token_cache;

use client::JkClient;
use redis::aio::ConnectionLike;

const APP_ID: &str = "9222001";

pub struct PayResult {
    pub success: bool,
    pub paid_amount: i64,
    pub order_status: Option<i64>,
    pub external_order_no: Option<String>,
    pub fail_reason: Option<String>,
}

pub fn calc_jk_payment_amount_fen(total_amount_fen: i64) -> i64 {
    payment_flow::calc_jk_payment_amount_fen(total_amount_fen)
}

pub async fn warmup<C>(
    redis: &mut C,
    seller_username: &str,
    seller_password: &str,
) -> Result<(), String>
where
    C: ConnectionLike + Send,
{
    let jk = JkClient::new()?;
    captcha::prime_ocr();
    let _ = token_cache::get_token(redis, &jk, seller_username, seller_password).await?;
    Ok(())
}

/// Execute the full JK health-card payment flow.
///
/// - `redis`: mutable reference to Redis connection (for token caching)
/// - `seller_username` / `seller_password`: from env
/// - `card_no`: user's ID card number (健康卡号)
/// - `card_password`: user-supplied payment password
/// - `total_amount_fen`: order total in 分, converted to 元 with 0.95 rounding
pub async fn jk_pay<C>(
    redis: &mut C,
    seller_username: &str,
    seller_password: &str,
    card_no: &str,
    card_password: &str,
    total_amount_fen: i64,
) -> PayResult
where
    C: ConnectionLike + Send,
{
    match payment_flow::do_jk_pay(
        redis,
        seller_username,
        seller_password,
        card_no,
        card_password,
        total_amount_fen,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => PayResult {
            success: false,
            paid_amount: 0,
            order_status: None,
            external_order_no: None,
            fail_reason: Some(e),
        },
    }
}

/// 单独查询健康卡账户余额。
///
/// 这里复用支付流程里的第三方接口，只返回余额，不执行扣款。
pub async fn query_health_card_balance<C>(
    redis: &mut C,
    seller_username: &str,
    seller_password: &str,
    card_no: &str,
    card_password: &str,
) -> Result<f64, String>
where
    C: ConnectionLike + Send,
{
    payment_flow::query_health_card_balance(
        redis,
        seller_username,
        seller_password,
        card_no,
        card_password,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::payment_flow::{calc_jk_payment_amount_fen, choose_health_card_balance};
    use super::*;
    use serde_json::json;
    use std::env;

    #[test]
    fn calc_jk_payment_amount_fen_matches_frontend_example() {
        assert_eq!(calc_jk_payment_amount_fen(59_900), 63_053);
        assert_eq!(calc_jk_payment_amount_fen(100), 105);
        assert_eq!(calc_jk_payment_amount_fen(1), 1);
    }

    async fn build_redis_connection() -> redis::aio::ConnectionManager {
        let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let redis_port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
        let redis_username = env::var("REDIS_USERNAME").unwrap_or_else(|_| "default".to_string());
        let redis_password = env::var("REDIS_PASSWORD").unwrap_or_default();
        let redis_url = if redis_password.is_empty() {
            if redis_username == "default" {
                format!("redis://{}:{}/0", redis_host, redis_port)
            } else {
                format!(
                    "redis://{}@{}:{}/0",
                    urlencoding::encode(&redis_username),
                    redis_host,
                    redis_port
                )
            }
        } else {
            format!(
                "redis://{}:{}@{}:{}/0",
                urlencoding::encode(&redis_username),
                urlencoding::encode(&redis_password),
                redis_host,
                redis_port
            )
        };

        let client = redis::Client::open(redis_url).expect("redis client");
        redis::aio::ConnectionManager::new(client)
            .await
            .expect("redis connection")
    }

    /// 端到端测试：登录 + 健康卡支付 0.01 元。
    /// 运行：cargo test test_jk_pay_001 -- --nocapture --ignored
    #[tokio::test]
    #[ignore]
    async fn test_jk_pay_001() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init();

        let mut redis = build_redis_connection().await;

        // 测试数据来自 jk_order.py 注释
        let result = jk_pay(
            &mut redis,
            &env::var("JK_TEST_SELLER_USERNAME")
                .or_else(|_| env::var("JK_SELLER_USERNAME"))
                .expect("JK_TEST_SELLER_USERNAME or JK_SELLER_USERNAME must be set"),
            &env::var("JK_TEST_SELLER_PASSWORD")
                .or_else(|_| env::var("JK_SELLER_PASSWORD"))
                .expect("JK_TEST_SELLER_PASSWORD or JK_SELLER_PASSWORD must be set"),
            "310115199011060935",
            "093538",
            1, // 1 分 = 0.01 元；内部会按 0.95 换算
        )
        .await;

        println!("[test] success={}", result.success);
        println!("[test] paid_amount={} 分", result.paid_amount);
        println!("[test] order_status={:?}", result.order_status);
        println!("[test] external_order_no={:?}", result.external_order_no);
        println!("[test] fail_reason={:?}", result.fail_reason);

        assert!(result.success, "支付失败: {:?}", result.fail_reason);
        assert!(result.paid_amount > 0, "实付金额应大于 0");
        assert!(result.external_order_no.is_some(), "应返回 jk.cn 订单号");
    }

    /// 端到端测试：按数据库中的“刘东升”实名账户查询健康卡余额。
    /// 运行：cargo test test_query_health_card_balance_for_liu_dongsheng -- --nocapture --ignored
    #[tokio::test]
    #[ignore]
    async fn test_query_health_card_balance_for_liu_dongsheng() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let database_url = database_url.replace("localhost", "127.0.0.1");
        let db = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("database connection");
        let mut redis = build_redis_connection().await;

        #[derive(sqlx::FromRow)]
        struct Row {
            openid: String,
            real_name: String,
            id_card_number: String,
            payment_password: String,
        }

        let user = sqlx::query_as::<_, Row>(
            "SELECT openid, real_name, id_card_number, payment_password \
             FROM wechat_users WHERE real_name = ? ORDER BY id DESC LIMIT 1",
        )
        .bind("刘东升")
        .fetch_optional(&db)
        .await
        .expect("query user")
        .expect("刘东升 user not found");

        println!(
            "[test] openid={} real_name={} id_card_masked={}",
            user.openid,
            user.real_name,
            if user.id_card_number.len() >= 8 {
                format!(
                    "{}********{}",
                    &user.id_card_number[..4],
                    &user.id_card_number[user.id_card_number.len() - 4..]
                )
            } else {
                user.id_card_number.clone()
            }
        );

        let balance = query_health_card_balance(
            &mut redis,
            &env::var("JK_TEST_SELLER_USERNAME")
                .or_else(|_| env::var("JK_SELLER_USERNAME"))
                .expect("JK_TEST_SELLER_USERNAME or JK_SELLER_USERNAME must be set"),
            &env::var("JK_TEST_SELLER_PASSWORD")
                .or_else(|_| env::var("JK_SELLER_PASSWORD"))
                .expect("JK_TEST_SELLER_PASSWORD or JK_SELLER_PASSWORD must be set"),
            &user.id_card_number,
            &user.payment_password,
        )
        .await
        .expect("health card balance query failed");

        println!("[test] health card balance = {:.2} 元", balance);
        assert!(balance >= 0.0, "balance should not be negative");
    }

    #[test]
    fn test_choose_health_card_balance_prefers_positive_health_card_account() {
        let channels = vec![
            json!({
                "accountName": "健康险直付",
                "balance": 0.0
            }),
            json!({
                "accountName": "养老险健康卡",
                "balance": 126.93
            }),
        ];

        let balance = choose_health_card_balance(&channels);
        assert_eq!(balance, Some(126.93));
    }
}
