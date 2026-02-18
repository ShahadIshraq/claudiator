use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::{check_auth, check_rate_limit, extract_client_ip, record_auth_failure};
use crate::db::queries;
use crate::error::AppError;
use crate::models::response::{DeviceListResponse, SessionListResponse};
use crate::router::AppState;

pub async fn list_devices_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<DeviceListResponse>, AppError> {
    let ip = extract_client_ip(&headers);
    check_rate_limit(&state.auth_failures, ip)?;
    if let Err(e) = check_auth(&headers, &state.api_key) {
        record_auth_failure(&state.auth_failures, ip);
        return Err(e);
    }

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
    headers: HeaderMap,
    Path(device_id): Path<String>,
    Query(params): Query<SessionQueryParams>,
) -> Result<Json<SessionListResponse>, AppError> {
    let ip = extract_client_ip(&headers);
    check_rate_limit(&state.auth_failures, ip)?;
    if let Err(e) = check_auth(&headers, &state.api_key) {
        record_auth_failure(&state.auth_failures, ip);
        return Err(e);
    }

    let limit = params.limit.unwrap_or(50);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    let sessions = queries::list_sessions(&conn, &device_id, params.status.as_deref(), limit)?;

    Ok(Json(SessionListResponse { sessions }))
}
