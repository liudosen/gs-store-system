use super::{crypto::calc_sig, APP_ID};
#[cfg(feature = "jk-ocr")]
use once_cell::sync::OnceCell;
use rand::Rng;
use serde_json::Value;
use std::collections::BTreeMap;

#[cfg(feature = "jk-ocr")]
static OCR: OnceCell<ddddocr::Ddddocr<'static>> = OnceCell::new();

#[cfg(feature = "jk-ocr")]
pub(super) fn prime_ocr() {
    let _ = get_ocr();
}

#[cfg(not(feature = "jk-ocr"))]
pub(super) fn prime_ocr() {
    tracing::info!("[JK OCR] disabled; using random captcha fallback");
}

#[cfg(feature = "jk-ocr")]
fn get_ocr() -> Option<&'static ddddocr::Ddddocr<'static>> {
    OCR.get_or_try_init(|| {
        ddddocr::ddddocr_classification().map_err(|e| {
            tracing::warn!("[JK OCR] init failed: {}", e);
            e
        })
    })
    .ok()
}

#[derive(Debug)]
struct CaptchaChallenge {
    img_url: String,
    key: String,
}

pub(super) struct SolvedCaptcha {
    pub key: String,
    pub text: String,
}

pub(super) async fn solve_login_captcha(client: &reqwest::Client) -> Result<SolvedCaptcha, String> {
    let challenge = request_login_captcha(client).await?;
    let img_bytes = client
        .get(&challenge.img_url)
        .send()
        .await
        .map_err(|e| format!("img download error: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("img bytes error: {e}"))?;

    let text = recognize_captcha(&img_bytes);
    Ok(SolvedCaptcha {
        key: challenge.key,
        text,
    })
}

async fn request_login_captcha(client: &reqwest::Client) -> Result<CaptchaChallenge, String> {
    let mut cap_params: BTreeMap<String, String> = BTreeMap::new();
    cap_params.insert("_mt".to_string(), "kylin.requestCaptcha".to_string());
    cap_params.insert("_sm".to_string(), "md5".to_string());
    cap_params.insert("_aid".to_string(), APP_ID.to_string());
    let sig = calc_sig(&cap_params, "");
    cap_params.insert("_sig".to_string(), sig);
    let cap_form: Vec<(String, String)> = cap_params.into_iter().collect();

    let cap_data: Value = client
        .post("https://api.jk.cn/m.api?_mt=kylin.requestCaptcha")
        .header("Origin", "https://www.jk.cn")
        .header("Referer", "https://www.jk.cn/")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded;charset=UTF-8",
        )
        .form(&cap_form)
        .send()
        .await
        .map_err(|e| format!("captcha request error: {e}"))?
        .json()
        .await
        .map_err(|e| format!("captcha parse error: {e}"))?;

    let content = cap_data
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .ok_or("no captcha content")?;

    let img_url = content
        .get("imgUrl")
        .and_then(|v| v.as_str())
        .ok_or("no imgUrl")?
        .to_string();
    let key = content
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or("no key")?
        .to_string();

    Ok(CaptchaChallenge { img_url, key })
}

fn recognize_captcha(img_bytes: &[u8]) -> String {
    #[cfg(feature = "jk-ocr")]
    {
        match get_ocr() {
            Some(ocr) => match ocr.classification(img_bytes) {
                Ok(text) => {
                    tracing::info!("[JK OCR] recognized: {}", text);
                    text
                }
                Err(e) => {
                    tracing::warn!(
                        "[JK OCR] classification failed: {}, using random fallback",
                        e
                    );
                    random_captcha()
                }
            },
            None => {
                tracing::warn!("[JK OCR] not available, using random fallback");
                random_captcha()
            }
        }
    }

    #[cfg(not(feature = "jk-ocr"))]
    {
        let _ = img_bytes;
        tracing::warn!("[JK OCR] disabled, using random captcha fallback");
        random_captcha()
    }
}

fn random_captcha() -> String {
    let mut rng = rand::thread_rng();
    (0..4)
        .map(|_| {
            let charset = b"abcdefghijklmnopqrstuvwxyz0123456789";
            charset[rng.gen_range(0..charset.len())] as char
        })
        .collect()
}
