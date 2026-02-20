use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::ReadAuth;
use crate::db::queries;
use crate::error::AppError;
use crate::models::response::{DeviceListResponse, SessionListResponse};
use crate::router::AppState;

pub async fn list_devices_handler(
    State(state): State<Arc<AppState>>,
    _auth: ReadAuth,
) -> Result<Json<DeviceListResponse>, AppError> {
    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    let devices = queries::list_devices(&conn)?;

    Ok(Json(DeviceListResponse { devices }))
}

#[derive(Deserialize)]
pub struct SessionQueryParams {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list_device_sessions_handler(
    State(state): State<Arc<AppState>>,
    _auth: ReadAuth,
    Path(device_id): Path<String>,
    Query(params): Query<SessionQueryParams>,
) -> Result<Json<SessionListResponse>, AppError> {
    let limit = params.limit.unwrap_or(50);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    let sessions = queries::list_sessions(&conn, &device_id, params.status.as_deref(), limit)?;

    Ok(Json(SessionListResponse { sessions }))
}
