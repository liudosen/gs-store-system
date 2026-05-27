use axum::{
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::domains::order::ws;

use crate::{
    common::api::Health,
    domains::{auth, customer, onboarding, order, veteran},
    infra::state::AppState,
};

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route(
            "/onboarding/veteran-join",
            post(onboarding::handler::veteran_join),
        )
        .route(
            "/customer/auth/send-sms-code",
            post(customer::handler::send_sms_code),
        )
        .route(
            "/customer/auth/login-by-sms",
            post(customer::handler::login_by_sms),
        )
        .route(
            "/customer/service-items",
            get(customer::handler::list_service_items),
        )
        .route("/customer/me", get(customer::handler::get_me))
        .route(
            "/customer/me/region",
            post(customer::handler::update_region),
        )
        .route(
            "/customer/addresses",
            get(customer::handler::list_addresses).post(customer::handler::create_address),
        )
        .route(
            "/customer/addresses/:id",
            post(customer::handler::update_address),
        )
        .route(
            "/customer/orders",
            get(order::handler::list_orders).post(order::handler::create_order),
        )
        .route("/customer/orders/:id", get(order::handler::get_order_detail))
        .route(
            "/veteran/orders/available",
            get(order::handler::list_available_orders),
        )
        .route(
            "/veteran/orders/:id/accept",
            post(order::handler::accept_order),
        )
        .route(
            "/veteran/orders/:id/cancel",
            post(order::handler::cancel_order),
        )
        .route(
            "/veteran/orders/assigned",
            get(order::handler::list_assigned_orders),
        )
        .route(
            "/veteran/orders/:id/detail",
            get(order::handler::get_assigned_order_detail),
        )
        .route("/veteran/auth/send-sms-code", post(auth::handler::send_sms_code))
        .route(
            "/veteran/auth/register-by-sms",
            post(auth::handler::register_by_sms_code),
        )
        .route("/veteran/me", get(veteran::handler::get_me))
        .route(
            "/veteran/me/region",
            post(veteran::handler::update_region),
        )
        .route(
            "/veteran/stats/daily",
            get(veteran::handler::get_daily_stats),
        )
        .route("/veteran/ws/orders", get(ws::ws_handler))
}

async fn health() -> impl IntoResponse {
    Json(Health {
        status: "ok",
        app: "gs-store-system",
    })
}
