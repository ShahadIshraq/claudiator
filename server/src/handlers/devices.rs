use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::check_auth;
use crate::db::queries;
use crate::error::AppError;
use crate::models::response::{DeviceListResponse, SessionListResponse};
use crate::router::AppState;

pub(crate) async fn list_devices_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<DeviceListResponse>, AppError> {
    check_auth(&headers, &state.api_key)?;

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    let devices = queries::list_devices(&conn)?;

    Ok(Json(DeviceListResponse { devices }))
}

#[derive(Deserialize)]
pub(crate) struct SessionQueryParams {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

pub(crate) async fn list_device_sessions_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
    Query(params): Query<SessionQueryParams>,
) -> Result<Json<SessionListResponse>, AppError> {
    check_auth(&headers, &state.api_key)?;

    let limit = params.limit.unwrap_or(50);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    let sessions = queries::list_sessions(&conn, &device_id, params.status.as_deref(), limit)?;

    Ok(Json(SessionListResponse { sessions }))
}
