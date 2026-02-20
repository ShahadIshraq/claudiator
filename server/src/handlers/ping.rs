use axum::extract::State;
use axum::Json;
use std::sync::Arc;

use crate::auth::ReadAuth;
use crate::error::AppError;
use crate::models::response::StatusOk;
use crate::router::AppState;

pub async fn ping_handler(
    State(state): State<Arc<AppState>>,
    _auth: ReadAuth,
) -> Result<Json<StatusOk>, AppError> {
    let data_v = state.version.load(std::sync::atomic::Ordering::Relaxed);
    let notif_v = state
        .notification_version
        .load(std::sync::atomic::Ordering::Relaxed);
    Ok(Json(StatusOk::with_versions(data_v, notif_v)))
}
