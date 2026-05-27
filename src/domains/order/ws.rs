use std::collections::HashMap;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::{broadcast, RwLock};

use crate::{
    common::api::ApiError,
    domains::{
        auth::service as auth_service,
        order::service as order_service,
        veteran::repository as veteran_repository,
    },
    infra::state::AppState,
};

pub struct OrderBroadcaster {
    channels: RwLock<HashMap<String, broadcast::Sender<String>>>,
}

impl OrderBroadcaster {
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
        }
    }

    pub async fn subscribe(&self, region_code: &str) -> broadcast::Receiver<String> {
        let mut channels = self.channels.write().await;
        channels
            .entry(region_code.to_string())
            .or_insert_with(|| broadcast::channel(32).0)
            .subscribe()
    }

    pub async fn broadcast(&self, region_code: &str, msg: String) {
        let channels = self.channels.read().await;
        if let Some(sender) = channels.get(region_code) {
            let _ = sender.send(msg);
        }
    }
}

#[derive(Deserialize)]
pub struct WsQuery {
    token: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let session = auth_service::resolve_veteran_token(&state, &query.token).await?;
    let profile = veteran_repository::find_veteran_by_id(&state.db, session.veteran_id)
        .await?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid session"))?;

    let region_code = profile.region_code.clone();

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, region_code)))
}

async fn handle_socket(socket: WebSocket, state: AppState, region_code: String) {
    let broadcaster = state.order_broadcaster.as_ref();
    let (mut sender, mut receiver) = socket.split();
    let mut subscriber = broadcaster.subscribe(&region_code).await;

    // 发送初始可用订单列表
    if let Ok(rows) =
        order_service::fetch_matching_orders(&state.db, &region_code).await
    {
        let msg = serde_json::json!({
            "type": "init",
            "orders": rows
        });
        let _ = sender.send(Message::Text(msg.to_string().into())).await;
    }

    // 双循环：接收 ws 消息（心跳/关闭） + 接收广播
    loop {
        tokio::select! {
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(_))) => {
                        let _ = sender.send(Message::Pong(vec![].into())).await;
                    }
                    _ => {}
                }
            }
            result = subscriber.recv() => {
                match result {
                    Ok(msg) => {
                        let _ = sender.send(Message::Text(msg.into())).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(region_code, skipped = n, "ws subscriber lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}
