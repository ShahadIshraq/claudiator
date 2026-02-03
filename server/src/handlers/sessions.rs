use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::check_auth;
use crate::db::queries;
use crate::error::AppError;
use crate::models::response::EventListResponse;
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
    check_auth(&headers, &state.api_key)?;

    let limit = params.limit.unwrap_or(100);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {}", e)))?;

    let events = queries::list_events(&conn, &session_id, limit)?;

    Ok(Json(EventListResponse { events }))
}
