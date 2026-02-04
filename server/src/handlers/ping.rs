use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use std::sync::Arc;

use crate::auth::check_auth;
use crate::error::AppError;
use crate::models::response::StatusOk;
use crate::router::AppState;

pub async fn ping_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<StatusOk>, AppError> {
    check_auth(&headers, &state.api_key)?;
    let data_v = state.version.load(std::sync::atomic::Ordering::Relaxed);
    let notif_v = state.notification_version.load(std::sync::atomic::Ordering::Relaxed);
    Ok(Json(StatusOk::with_versions(data_v, notif_v)))
}
