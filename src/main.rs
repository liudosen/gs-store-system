use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    messages: Arc<Mutex<Vec<Message>>>,
}

#[derive(Clone, Serialize)]
struct Message {
    id: usize,
    text: String,
}

#[derive(Deserialize)]
struct CreateMessage {
    text: String,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    app: &'static str,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("rust_coupled_fullstack=debug,tower_http=debug")
        .init();

    let state = AppState {
        messages: Arc::new(Mutex::new(vec![Message {
            id: 1,
            text: "Hello from Rust API".to_string(),
        }])),
    };

    let api = Router::new()
        .route("/health", get(health))
        .route("/messages", get(list_messages).post(create_message));

    let app = Router::new()
        .nest("/api", api)
        .fallback_service(ServeDir::new("frontend").append_index_html_on_directories(true))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind server");

    println!("App running at http://{addr}");
    axum::serve(listener, app).await.expect("server failed");
}

async fn health() -> impl IntoResponse {
    Json(Health {
        status: "ok",
        app: "rust-coupled-fullstack",
    })
}

async fn list_messages(State(state): State<AppState>) -> impl IntoResponse {
    let messages = state.messages.lock().expect("messages lock poisoned");
    Json(messages.clone())
}

async fn create_message(
    State(state): State<AppState>,
    Json(payload): Json<CreateMessage>,
) -> impl IntoResponse {
    let mut messages = state.messages.lock().expect("messages lock poisoned");
    let id = messages.len() + 1;
    let message = Message {
        id,
        text: payload.text.trim().to_string(),
    };
    messages.push(message.clone());

    Json(message)
}
