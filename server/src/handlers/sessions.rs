use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::{check_auth, check_rate_limit, extract_client_ip, record_auth_failure};
use crate::db::queries;
use crate::error::AppError;
use crate::models::response::{EventListResponse, SessionListResponse};
use crate::router::AppState;

#[derive(Deserialize)]
pub struct EventQueryParams {
    pub limit: Option<i64>,
}

pub async fn list_session_events_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Query(params): Query<EventQueryParams>,
) -> Result<Json<EventListResponse>, AppError> {
    let ip = extract_client_ip(&headers);
    check_rate_limit(&state.auth_failures, ip)?;
    if let Err(e) = check_auth(&headers, &state.api_key) {
        record_auth_failure(&state.auth_failures, ip);
        return Err(e);
    }

    let limit = params.limit.unwrap_or(100);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    let events = queries::list_events(&conn, &session_id, limit)?;

    Ok(Json(EventListResponse { events }))
}

pub async fn list_all_sessions_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<super::devices::SessionQueryParams>,
) -> Result<Json<SessionListResponse>, AppError> {
    let ip = extract_client_ip(&headers);
    check_rate_limit(&state.auth_failures, ip)?;
    if let Err(e) = check_auth(&headers, &state.api_key) {
        record_auth_failure(&state.auth_failures, ip);
        return Err(e);
    }

    let limit = params.limit.unwrap_or(200);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    let sessions = queries::list_all_sessions(&conn, params.status.as_deref(), limit)?;

    Ok(Json(SessionListResponse { sessions }))
}
