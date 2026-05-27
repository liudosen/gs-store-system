use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Deserialize;
use sha1::Sha1;
use std::time::Duration;
use tracing::{info, warn};
use urlencoding::encode;
use uuid::Uuid;

use crate::infra::config::SmsConfig;

#[derive(Clone)]
pub struct SmsService {
    client: Client,
    endpoint: String,
    access_key_id: String,
    access_key_secret: String,
    sign_name: String,
    template_code: String,
    api_version: &'static str,
}

#[derive(Deserialize)]
struct AliyunSmsResponse {
    #[serde(rename = "Code")]
    code: String,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "BizId")]
    biz_id: Option<String>,
    #[serde(rename = "RequestId")]
    request_id: Option<String>,
}

impl SmsService {
    pub fn from_config(config: &SmsConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .context("failed to build sms http client")?;

        Ok(Self {
            client,
            endpoint: config.endpoint.clone(),
            access_key_id: config.access_key_id.clone(),
            access_key_secret: config.access_key_secret.clone(),
            sign_name: config.sign_name.clone(),
            template_code: config.template_code.clone(),
            api_version: config.api_version,
        })
    }

    pub async fn send_code(&self, phone: &str, code: &str) -> Result<()> {
        let template_param = serde_json::json!({ "code": code }).to_string();
        let mut params = vec![
            ("AccessKeyId".to_string(), self.access_key_id.clone()),
            ("Action".to_string(), "SendSms".to_string()),
            ("Format".to_string(), "JSON".to_string()),
            ("PhoneNumbers".to_string(), phone.to_string()),
            ("RegionId".to_string(), "cn-hangzhou".to_string()),
            ("SignName".to_string(), self.sign_name.clone()),
            ("SignatureMethod".to_string(), "HMAC-SHA1".to_string()),
            ("SignatureNonce".to_string(), Uuid::new_v4().to_string()),
            ("SignatureVersion".to_string(), "1.0".to_string()),
            ("TemplateCode".to_string(), self.template_code.clone()),
            ("TemplateParam".to_string(), template_param),
            (
                "Timestamp".to_string(),
                Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            ),
            ("Version".to_string(), self.api_version.to_string()),
        ];

        let signature = sign_aliyun_rpc_query(&params, &self.access_key_secret)?;
        params.push(("Signature".to_string(), signature));
        params.sort_by(|a, b| a.0.cmp(&b.0));

        let response = self
            .client
            .get(&self.endpoint)
            .query(&params)
            .send()
            .await
            .context("failed to request aliyun sms api")?;

        let status = response.status();
        let body = response
            .text()
            .await
            .context("failed to read aliyun sms response")?;

        if !status.is_success() {
            warn!(phone, status = %status, body, "aliyun sms http error");
            return Err(anyhow!("aliyun sms request failed with status {status}"));
        }

        let payload: AliyunSmsResponse =
            serde_json::from_str(&body).context("failed to parse aliyun sms response")?;

        if payload.code != "OK" {
            warn!(
                phone,
                code = payload.code,
                message = payload.message.clone().unwrap_or_default(),
                request_id = payload.request_id.clone().unwrap_or_default(),
                "aliyun sms business error"
            );
            return Err(anyhow!(
                "aliyun sms send failed: {} ({})",
                payload
                    .message
                    .unwrap_or_else(|| "unknown error".to_string()),
                payload.code
            ));
        }

        info!(
            phone,
            biz_id = payload.biz_id.unwrap_or_default(),
            request_id = payload.request_id.unwrap_or_default(),
            "aliyun sms verification code sent"
        );

        Ok(())
    }
}

fn sign_aliyun_rpc_query(params: &[(String, String)], secret: &str) -> Result<String> {
    let mut sorted = params.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let canonical_query = sorted
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                aliyun_percent_encode(key),
                aliyun_percent_encode(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&");

    let string_to_sign = format!("GET&%2F&{}", aliyun_percent_encode(&canonical_query));
    let mut mac =
        Hmac::<Sha1>::new_from_slice(format!("{secret}&").as_bytes()).context("invalid sms key")?;
    mac.update(string_to_sign.as_bytes());
    Ok(BASE64_STANDARD.encode(mac.finalize().into_bytes()))
}

fn aliyun_percent_encode(value: &str) -> String {
    encode(value)
        .replace('+', "%20")
        .replace('*', "%2A")
        .replace("%7E", "~")
}
