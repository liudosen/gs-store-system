use crate::error::AppError;
use crate::models::WechatUser;
use crate::routes::ApiResponse;
use crate::state::{AppState, MiniAppAuthConfig};
use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

const WECHAT_SESSION_TTL_SECONDS: i64 = 30 * 24 * 3600;
const WECHAT_TOKEN_PREFIX: &str = "welfare:wechat:token:";

static DEV_TOKEN_STORE: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Deserialize)]
pub struct WechatLoginRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct WechatCode2SessionResponse {
    pub openid: Option<String>,
    pub session_key: Option<String>,
    pub unionid: Option<String>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WechatLoginResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub token_type: String,
    pub user: WechatUserInfo,
}

#[derive(Debug, Serialize)]
pub struct WechatUserInfo {
    pub id: u64,
    pub openid: String,
    pub real_name: String,
    pub avatar_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WechatClaims {
    pub sub: String,
    pub wechat_id: u64,
    pub openid: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn wechat_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WechatLoginRequest>,
) -> Result<Json<ApiResponse<WechatLoginResponse>>, AppError> {
    tracing::info!(
        "Wechat login attempt received (code redacted, len={})",
        payload.code.len()
    );

    let openid = resolve_login_openid(&state, &payload.code).await?;

    tracing::info!("Wechat login success");

    let user = bootstrap_wechat_user(&state, &openid).await?;
    mark_wechat_user_login(&state, user.id).await?;

    let token = create_wechat_token(&state, user.id, &openid)?;
    store_wechat_session_token(&state, &token, user.id, &openid).await?;

    Ok(Json(ApiResponse::success(WechatLoginResponse {
        access_token: token,
        expires_in: WECHAT_SESSION_TTL_SECONDS,
        token_type: "Bearer".to_string(),
        user: wechat_user_info_from(user),
    })))
}

fn wechat_auth_config(state: &AppState) -> MiniAppAuthConfig<'_> {
    state.mini_app_auth_config()
}

async fn resolve_login_openid(state: &AppState, code: &str) -> Result<String, AppError> {
    let auth = wechat_auth_config(state);

    if let Some(dev_openid) = auth.dev_wechat_openid {
        tracing::warn!("DEV_WECHAT_OPENID is set, skipping WeChat code exchange");
        return Ok(dev_openid.to_string());
    }

    let wechat_resp = request_wechat_code2session(auth, code).await?;
    resolve_wechat_openid(wechat_resp)
}

async fn request_wechat_code2session(
    auth: MiniAppAuthConfig<'_>,
    code: &str,
) -> Result<WechatCode2SessionResponse, AppError> {
    let wechat_url = format!(
        "https://api.weixin.qq.com/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
        auth.wechat_appid, auth.wechat_secret, code
    );

    let client = reqwest::Client::new();
    let response = client.get(&wechat_url).send().await.map_err(|e| {
        tracing::error!("Failed to call wechat api: {}", e);
        AppError::InternalError("Failed to call wechat api".to_string())
    })?;

    response.json().await.map_err(|e| {
        tracing::error!("Failed to parse wechat response: {}", e);
        AppError::InternalError("Failed to parse wechat response".to_string())
    })
}

fn resolve_wechat_openid(resp: WechatCode2SessionResponse) -> Result<String, AppError> {
    let WechatCode2SessionResponse {
        openid,
        errcode,
        errmsg,
        ..
    } = resp;

    if let Some(errcode) = errcode {
        if errcode != 0 {
            let errmsg = errmsg.unwrap_or_default();
            tracing::error!("Wechat api error: {} - {}", errcode, errmsg);
            return Err(AppError::BadRequest(format!(
                "Wechat api error: {}",
                errmsg
            )));
        }
    }

    openid.ok_or_else(|| {
        tracing::error!("Wechat response missing openid");
        AppError::InternalError("Wechat response missing openid".to_string())
    })
}

async fn bootstrap_wechat_user(state: &AppState, openid: &str) -> Result<WechatUser, AppError> {
    let existing = sqlx::query_as::<_, WechatUser>("SELECT * FROM wechat_users WHERE openid = ?")
        .bind(openid)
        .fetch_optional(&state.db)
        .await?;

    if let Some(user) = existing {
        return Ok(user);
    }

    sqlx::query(
        r#"
        INSERT INTO wechat_users (openid, real_name, avatar_url, phone, country, province, city, gender, status)
        VALUES (?, '', '', '', '', '', '', 0, 1)
        "#,
    )
    .bind(openid)
    .execute(&state.db)
    .await?;

    let user = sqlx::query_as::<_, WechatUser>(
        "SELECT * FROM wechat_users WHERE openid = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(openid)
    .fetch_one(&state.db)
    .await?;

    tracing::info!("Created new wechat user with id: {}", user.id);
    Ok(user)
}

async fn mark_wechat_user_login(state: &AppState, user_id: u64) -> Result<(), AppError> {
    sqlx::query("UPDATE wechat_users SET last_login_at = NOW() WHERE id = ?")
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(())
}

fn create_wechat_token(state: &AppState, wechat_id: u64, openid: &str) -> Result<String, AppError> {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let auth = wechat_auth_config(state);
    let now = Utc::now();
    let exp = now + Duration::days(30);

    let claims = WechatClaims {
        sub: openid.to_string(),
        wechat_id,
        openid: openid.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(auth.jwt_secret.as_bytes()),
    )?;

    Ok(token)
}

async fn store_wechat_session_token(
    state: &AppState,
    token: &str,
    user_id: u64,
    openid: &str,
) -> Result<(), AppError> {
    if wechat_auth_config(state).dev_wechat_openid.is_some() {
        cache_dev_token(token, openid);
        return Ok(());
    }

    let wechat_key = wechat_token_key(token);
    let mut redis = state.redis_conn().await?;
    redis::cmd("SETEX")
        .arg(&wechat_key)
        .arg(WECHAT_SESSION_TTL_SECONDS)
        .arg(user_id.to_string())
        .query_async::<_, ()>(&mut redis)
        .await
        .map_err(|e| {
            tracing::error!("Redis error storing wechat token: {}", e);
            AppError::InternalError("Failed to store session".to_string())
        })?;

    Ok(())
}

pub async fn get_openid_from_token(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Option<String> {
    resolve_openid_from_headers(state, headers).await.ok()
}

pub async fn validate_wechat_user(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<String, AppError> {
    resolve_openid_from_headers(state, headers).await
}

async fn resolve_openid_from_headers(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<String, AppError> {
    let token = bearer_token(headers)?;
    validate_cached_session(state, token).await?;
    decode_wechat_claims(state, token).map(|claims| claims.openid)
}

fn bearer_token(headers: &axum::http::HeaderMap) -> Result<&str, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::InvalidToken)?;

    auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::InvalidToken)
}

async fn validate_cached_session(state: &AppState, token: &str) -> Result<(), AppError> {
    if wechat_auth_config(state).dev_wechat_openid.is_some() {
        let store = DEV_TOKEN_STORE
            .lock()
            .map_err(|_| AppError::InternalError("Failed to check session".to_string()))?;

        if store.contains_key(token) {
            return Ok(());
        }

        return Err(AppError::WechatTokenExpired);
    }

    let wechat_key = wechat_token_key(token);
    let mut redis = state.redis_conn().await?;
    let exists: bool = redis::cmd("EXISTS")
        .arg(&wechat_key)
        .query_async(&mut redis)
        .await
        .map_err(|e| {
            tracing::error!("Redis error checking wechat token: {}", e);
            AppError::InternalError("Failed to check session".to_string())
        })?;

    if exists {
        Ok(())
    } else {
        Err(AppError::WechatTokenExpired)
    }
}

fn decode_wechat_claims(state: &AppState, token: &str) -> Result<WechatClaims, AppError> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    let auth = wechat_auth_config(state);
    decode::<WechatClaims>(
        token,
        &DecodingKey::from_secret(auth.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::InvalidToken)
}

fn wechat_token_key(token: &str) -> String {
    format!("{}{}", WECHAT_TOKEN_PREFIX, token)
}

fn cache_dev_token(token: &str, openid: &str) {
    match DEV_TOKEN_STORE.lock() {
        Ok(mut store) => {
            store.insert(token.to_string(), openid.to_string());
        }
        Err(_) => {
            tracing::error!("dev token store poisoned, skipping token cache");
        }
    }
}

fn wechat_user_info_from(user: WechatUser) -> WechatUserInfo {
    WechatUserInfo {
        id: user.id,
        openid: user.openid,
        real_name: user.real_name,
        avatar_url: user.avatar_url,
    }
}

pub async fn check_my_id_card(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ApiResponse<Option<String>>>, AppError> {
    let openid = get_openid_from_token(&state, &headers)
        .await
        .ok_or(AppError::WechatTokenExpired)?;

    let id_card: Option<String> =
        sqlx::query_scalar("SELECT id_card_number FROM wechat_users WHERE openid = ?")
            .bind(&openid)
            .fetch_optional(&state.db)
            .await?;

    let result = id_card.filter(|s| !s.is_empty());

    Ok(Json(ApiResponse::success(result)))
}

pub async fn get_my_userinfo(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ApiResponse<WechatUser>>, AppError> {
    let openid = get_openid_from_token(&state, &headers)
        .await
        .ok_or(AppError::WechatTokenExpired)?;

    let user = sqlx::query_as::<_, WechatUser>("SELECT * FROM wechat_users WHERE openid = ?")
        .bind(&openid)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("User".to_string()))?;

    Ok(Json(ApiResponse::success(user)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateMyUserRequest {
    pub real_name: Option<String>,
    pub phone: Option<String>,
    pub id_card_number: Option<String>,
}

pub async fn update_my_userinfo(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<UpdateMyUserRequest>,
) -> Result<Json<ApiResponse<WechatUser>>, AppError> {
    let openid = get_openid_from_token(&state, &headers)
        .await
        .ok_or(AppError::WechatTokenExpired)?;

    let existing = sqlx::query_as::<_, WechatUser>("SELECT * FROM wechat_users WHERE openid = ?")
        .bind(&openid)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("User".to_string()))?;

    let mut updates: Vec<&str> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    if let Some(ref v) = payload.real_name {
        updates.push("real_name = ?");
        values.push(v.clone());
    }
    if let Some(ref v) = payload.phone {
        updates.push("phone = ?");
        values.push(v.clone());
    }
    if let Some(ref v) = payload.id_card_number {
        updates.push("id_card_number = ?");
        values.push(v.clone());
    }

    if updates.is_empty() {
        return Ok(Json(ApiResponse::success(existing)));
    }

    let query = format!(
        "UPDATE wechat_users SET {} WHERE openid = ?",
        updates.join(", ")
    );

    let mut q = sqlx::query(&query);
    for v in &values {
        q = q.bind(v);
    }
    q = q.bind(&openid);
    q.execute(&state.db).await?;

    let user = sqlx::query_as::<_, WechatUser>("SELECT * FROM wechat_users WHERE openid = ?")
        .bind(&openid)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(ApiResponse::success(user)))
}
