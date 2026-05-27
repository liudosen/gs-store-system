use redis::aio::ConnectionManager;
use sqlx::MySqlPool;
use std::sync::Arc;

use crate::{domains::order::ws::OrderBroadcaster, infra::sms::SmsService};

#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    pub redis: ConnectionManager,
    pub sms_service: Option<Arc<SmsService>>,
    pub order_broadcaster: Arc<OrderBroadcaster>,
}

impl AppState {
    pub fn new(
        db: MySqlPool,
        redis: ConnectionManager,
        sms_service: Option<Arc<SmsService>>,
    ) -> Self {
        Self {
            db,
            redis,
            sms_service,
            order_broadcaster: Arc::new(OrderBroadcaster::new()),
        }
    }
}
