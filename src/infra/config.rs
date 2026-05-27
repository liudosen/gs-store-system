use anyhow::{Context, Result};
use std::{env, net::IpAddr, net::SocketAddr};

const ALIYUN_SMS_API_VERSION: &str = "2017-05-25";
const ALIYUN_SMS_DEFAULT_ENDPOINT: &str = "https://dysmsapi.aliyuncs.com/";

pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub server_host: IpAddr,
    pub server_port: u16,
    pub sms: SmsConfig,
}

pub struct SmsConfig {
    pub endpoint: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub sign_name: String,
    pub template_code: String,
    pub api_version: &'static str,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            redis_url: env::var("REDIS_URL").context("REDIS_URL is required")?,
            server_host: env::var("SERVER_HOST")
                .ok()
                .and_then(|value| value.parse::<IpAddr>().ok())
                .unwrap_or(IpAddr::from([127, 0, 0, 1])),
            server_port: env::var("APP_PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(9000),
            sms: SmsConfig::from_env()?,
        })
    }

    pub fn server_addr(&self) -> SocketAddr {
        SocketAddr::from((self.server_host, self.server_port))
    }
}

impl SmsConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            endpoint: env::var("ALIYUN_SMS_ENDPOINT")
                .unwrap_or_else(|_| ALIYUN_SMS_DEFAULT_ENDPOINT.to_string()),
            access_key_id: env::var("ALIYUN_SMS_ACCESS_KEY_ID")
                .or_else(|_| env::var("ALIYUN_ACCESS_KEY_ID"))
                .context("ALIYUN_SMS_ACCESS_KEY_ID is required")?,
            access_key_secret: env::var("ALIYUN_SMS_ACCESS_KEY_SECRET")
                .or_else(|_| env::var("ALIYUN_ACCESS_KEY_SECRET"))
                .context("ALIYUN_SMS_ACCESS_KEY_SECRET is required")?,
            sign_name: env::var("ALIYUN_SMS_SIGN_NAME")
                .context("ALIYUN_SMS_SIGN_NAME is required")?,
            template_code: env::var("ALIYUN_SMS_TEMPLATE_CODE")
                .context("ALIYUN_SMS_TEMPLATE_CODE is required")?,
            api_version: ALIYUN_SMS_API_VERSION,
        })
    }
}
