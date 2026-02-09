use axum::routing::{get, post};
use axum::Router;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use crate::apns::ApnsClient;
use crate::db::pool::DbPool;
use crate::handlers;

pub struct AppState {
    pub api_key: String,
    pub db_pool: DbPool,
    pub version: AtomicU64,
    pub notification_version: AtomicU64,
    pub apns_client: Option<Arc<ApnsClient>>,
}

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/ping", get(handlers::ping::ping_handler))
        .route("/api/v1/events", post(handlers::events::events_handler))
        .route(
            "/api/v1/devices",
            get(handlers::devices::list_devices_handler),
        )
        .route(
            "/api/v1/devices/:device_id/sessions",
            get(handlers::devices::list_device_sessions_handler),
        )
        .route(
            "/api/v1/sessions",
            get(handlers::sessions::list_all_sessions_handler),
        )
        .route(
            "/api/v1/sessions/:session_id/events",
            get(handlers::sessions::list_session_events_handler),
        )
        .route(
            "/api/v1/push/register",
            post(handlers::push::push_register_handler),
        )
        .route(
            "/api/v1/notifications",
            get(handlers::notifications::list_notifications_handler),
        )
        .with_state(state)
}
