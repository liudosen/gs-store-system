use super::{captcha, crypto, APP_ID};
use once_cell::sync::OnceCell;
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Duration;

const MAX_LOGIN_RETRY: u32 = 10;

static HTTP_CLIENT: OnceCell<reqwest::Client> = OnceCell::new();

pub(super) struct JkClient {
    client: reqwest::Client,
}

impl JkClient {
    pub(super) fn new() -> Result<Self, String> {
        Ok(Self {
            client: get_http_client()?.clone(),
        })
    }

    pub(super) async fn api(
        &self,
        mt: &str,
        extra: &[(&str, &str)],
        wtk: &str,
    ) -> Result<Value, String> {
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("_mt".to_string(), mt.to_string());
        params.insert("_sm".to_string(), "md5".to_string());
        params.insert("_aid".to_string(), APP_ID.to_string());
        for (k, v) in extra {
            params.insert(k.to_string(), v.to_string());
        }
        if !wtk.is_empty() {
            params.insert("_wtk".to_string(), wtk.to_string());
        }
        let sig = crypto::calc_sig(&params, wtk);
        params.insert("_sig".to_string(), sig);

        let form: Vec<(String, String)> = params.into_iter().collect();
        let url = format!("https://api.jk.cn/m.api?_mt={mt}");

        self.form_request(&url)
            .form(&form)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {e}"))?
            .json::<Value>()
            .await
            .map_err(|e| format!("JSON parse error: {e}"))
    }

    pub(super) async fn login(&self, username: &str, password: &str) -> Result<String, String> {
        let pwd_hashed = crypto::pwd_hash(password);

        for attempt in 1..=MAX_LOGIN_RETRY {
            let captcha = captcha::solve_login_captcha(&self.client).await?;

            tracing::info!(
                "[JK Login] attempt {}/{} captcha={}",
                attempt,
                MAX_LOGIN_RETRY,
                captcha.text
            );

            let resp: Value = self
                .form_request("https://jk.cn/login/loginname")
                .form(&[
                    ("loginName", username),
                    ("password", &pwd_hashed),
                    ("captcha", &captcha.text),
                    ("_cap", &captcha.key),
                    ("appId", APP_ID),
                ])
                .send()
                .await
                .map_err(|e| format!("login request error: {e}"))?
                .json()
                .await
                .map_err(|e| format!("login parse error: {e}"))?;

            if resp.get("success").and_then(|v| v.as_bool()) == Some(true) {
                let wtk = resp
                    .pointer("/model/_wtk")
                    .and_then(|v| v.as_str())
                    .ok_or("no wtk in response")?
                    .to_string();
                tracing::info!("[JK Login] success wtk={}", wtk);
                return Ok(wtk);
            }

            let err_code = resp
                .get("errorCode")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let err_msg = resp
                .get("errorMessage")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            tracing::warn!("[JK Login] failed: {} {}", err_code, err_msg);
        }
        Err("登录失败，已超过最大重试次数".to_string())
    }

    fn form_request(&self, url: &str) -> reqwest::RequestBuilder {
        self.client
            .post(url)
            .header("Origin", "https://www.jk.cn")
            .header("Referer", "https://www.jk.cn/")
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded;charset=UTF-8",
            )
    }
}

fn get_http_client() -> Result<&'static reqwest::Client, String> {
    HTTP_CLIENT
        .get_or_try_init(|| {
            reqwest::Client::builder()
                .cookie_store(true)
                .pool_idle_timeout(Duration::from_secs(90))
                .pool_max_idle_per_host(8)
                .user_agent(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                     AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36",
                )
                .build()
                .map_err(|e| {
                    tracing::warn!("[JK HTTP] init failed: {}", e);
                    format!("HTTP client error: {e}")
                })
        })
        .map(|client| client)
}
