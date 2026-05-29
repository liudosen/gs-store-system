use crate::error::AppError;
use deadpool_redis::{Connection, Pool};
use sqlx::MySqlPool;

#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    pub redis: Pool,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub auth_require_redis_session: bool,
    #[allow(dead_code)]
    pub bcrypt_cost: u32,
    pub wechat_appid: String,
    pub wechat_secret: String,
    pub dev_wechat_openid: Option<String>,
    pub jk_seller_username: String,
    pub jk_seller_password: String,
    pub oss_endpoint: String,
    pub oss_access_key_id: String,
    pub oss_access_key_secret: String,
    pub oss_bucket: String,
    pub oss_domain: String,
}

#[derive(Clone, Copy)]
pub struct MiniAppAuthConfig<'a> {
    pub wechat_appid: &'a str,
    pub wechat_secret: &'a str,
    pub dev_wechat_openid: Option<&'a str>,
    pub jwt_secret: &'a str,
}

impl AppState {
    pub async fn redis_conn(&self) -> Result<Connection, AppError> {
        self.redis.get().await.map_err(|error| {
            AppError::InternalError(format!("Failed to get Redis connection from pool: {error}"))
        })
    }

    pub fn mini_app_auth_config(&self) -> MiniAppAuthConfig<'_> {
        MiniAppAuthConfig {
            wechat_appid: &self.wechat_appid,
            wechat_secret: &self.wechat_secret,
            dev_wechat_openid: self.dev_wechat_openid.as_deref(),
            jwt_secret: &self.jwt_secret,
        }
    }
}
