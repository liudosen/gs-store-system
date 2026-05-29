use super::client::JkClient;
use redis::aio::ConnectionLike;
use redis::AsyncCommands;

/// Redis key for cached wtk token (expires 8 hours)
const TOKEN_REDIS_KEY: &str = "welfare:jk:wtk";
const TOKEN_TTL_SECS: u64 = 8 * 3600;

pub(super) fn should_clear_cached_token(err: &str) -> bool {
    let msg = err.to_lowercase();
    msg.contains("wtk")
        || msg.contains("token")
        || msg.contains("登录")
        || msg.contains("登陆")
        || msg.contains("expired")
        || msg.contains("失效")
        || msg.contains("无效")
}

pub(super) async fn clear_cached_token<C>(redis: &mut C)
where
    C: ConnectionLike + Send,
{
    let _: () = redis.del(TOKEN_REDIS_KEY).await.unwrap_or(());
}

pub(super) async fn get_token<C>(
    redis: &mut C,
    jk: &JkClient,
    username: &str,
    password: &str,
) -> Result<String, String>
where
    C: ConnectionLike + Send,
{
    let cached: Option<String> = redis
        .get(TOKEN_REDIS_KEY)
        .await
        .map_err(|e| format!("Redis get error: {e}"))?;

    if let Some(wtk) = cached {
        tracing::info!("[JK Token] using cached wtk from Redis (skip verify)");
        return Ok(wtk);
    }

    tracing::info!("[JK Token] no cached token, logging in");
    let wtk = jk.login(username, password).await?;

    let _: () = redis
        .set_ex(TOKEN_REDIS_KEY, &wtk, TOKEN_TTL_SECS)
        .await
        .map_err(|e| format!("Redis set error: {e}"))?;
    tracing::info!("[JK Token] token cached in Redis for {}s", TOKEN_TTL_SECS);

    Ok(wtk)
}
