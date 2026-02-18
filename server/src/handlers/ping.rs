use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use std::sync::Arc;

use crate::auth::{check_auth, check_rate_limit, extract_client_ip, record_auth_failure};
use crate::error::AppError;
use crate::models::response::StatusOk;
use crate::router::AppState;

pub async fn ping_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<StatusOk>, AppError> {
    let ip = extract_client_ip(&headers);
    check_rate_limit(&state.auth_failures, ip)?;
    if let Err(e) = check_auth(&headers, &state.api_key) {
        record_auth_failure(&state.auth_failures, ip);
        return Err(e);
    }
    let data_v = state.version.load(std::sync::atomic::Ordering::Relaxed);
    let notif_v = state
        .notification_version
        .load(std::sync::atomic::Ordering::Relaxed);
    Ok(Json(StatusOk::with_versions(data_v, notif_v)))
}
