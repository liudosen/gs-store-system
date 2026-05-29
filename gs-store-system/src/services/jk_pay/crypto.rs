use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use md5::Digest;
use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Encrypt, RsaPublicKey};
use serde_json::json;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

const RSA_PUBLIC_KEY: &str = concat!(
    "MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDKzDDsrhcP7iRsbbVhn30P/38R",
    "+b4DNmV0bhrxG7lm1kBdhk8+br7g42JCK5m7Vs50FWnSXWSkNoKT+fuzg23x3WpR",
    "xu6s84FSFj9Un6H4eRFSAOKyxTQuNftr4RYDFvkRsHlGGnhiHv7dXgufD7TfaTNr",
    "fI/K4pLZRhfzcqHecwIDAQAB"
);

pub(super) fn md5_hex(s: &str) -> String {
    let mut hasher = md5::Md5::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(super) fn calc_sig(params: &BTreeMap<String, String>, wtk: &str) -> String {
    let mut parts = String::new();
    for (k, v) in params.iter() {
        if k == "_sig" {
            continue;
        }
        parts.push_str(k);
        parts.push('=');
        parts.push_str(v);
    }
    if !wtk.is_empty() {
        parts.push_str(wtk);
    } else {
        parts.push_str("jk.pingan.com");
    }
    md5_hex(&parts)
}

pub(super) fn pwd_hash(password: &str) -> String {
    md5_hex(&format!("{}pajk.cn", password))
}

fn public_key_pem() -> String {
    let body = RSA_PUBLIC_KEY
        .chars()
        .collect::<Vec<_>>()
        .chunks(64)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
        body
    )
}

pub(super) fn rsa_encrypt(plain: &str) -> Result<String, String> {
    let pem = public_key_pem();
    let public_key =
        RsaPublicKey::from_public_key_pem(&pem).map_err(|e| format!("RSA key error: {e}"))?;
    let mut rng = rand::thread_rng();
    let encrypted = public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, plain.as_bytes())
        .map_err(|e| format!("RSA encrypt error: {e}"))?;
    Ok(BASE64.encode(&encrypted))
}

pub(super) fn make_card_password(card_no: &str, card_password: &str) -> Result<String, String> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let payload = json!({
        "cardNo": card_no,
        "pd": card_password,
        "timestamp": ts
    });
    let payload_str = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    rsa_encrypt(&payload_str)
}
