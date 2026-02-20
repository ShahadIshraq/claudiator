use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::Router;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::apns::ApnsClient;
use crate::auth::AuthFailureMap;
use crate::db::pool::DbPool;
use crate::handlers;

pub struct AppState {
    pub master_key: String,
    pub db_pool: DbPool,
    pub version: AtomicU64,
    pub notification_version: AtomicU64,
    pub last_cleanup: AtomicU64,
    pub apns_client: Option<Arc<ApnsClient>>,
    pub retention_events_days: u64,
    pub retention_sessions_days: u64,
    pub retention_devices_days: u64,
    pub auth_failures: Arc<AuthFailureMap>,
}

/// Converts a tower timeout error into an HTTP 408 Request Timeout response.
async fn handle_timeout_error(err: tower::BoxError) -> (StatusCode, &'static str) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "Request timed out")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    }
}

pub fn build_router(state: Arc<AppState>) -> Router {
    let admin_router = Router::new()
        .route(
            "/api-keys",
            post(handlers::admin::create_api_key_handler)
                .get(handlers::admin::list_api_keys_handler),
        )
        .route(
            "/api-keys/:id",
            delete(handlers::admin::delete_api_key_handler),
        );

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
        .route(
            "/api/v1/notifications/ack",
            post(handlers::notifications::acknowledge_notifications_handler),
        )
        .nest("/admin", admin_router)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(30)))
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(state)
}
