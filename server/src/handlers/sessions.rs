use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::ReadAuth;
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
    _auth: ReadAuth,
    Path(session_id): Path<String>,
    Query(params): Query<EventQueryParams>,
) -> Result<Json<EventListResponse>, AppError> {
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
    _auth: ReadAuth,
    Query(params): Query<super::devices::SessionQueryParams>,
) -> Result<Json<SessionListResponse>, AppError> {
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0).max(0);
    let exclude_ended = params.exclude_ended.unwrap_or(false);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    let result = queries::list_all_sessions_paginated(
        &conn,
        params.status.as_deref(),
        exclude_ended,
        limit,
        offset,
    )?;

    Ok(Json(SessionListResponse {
        sessions: result.sessions,
        has_more: result.has_more,
        next_offset: result.next_offset,
    }))
}
