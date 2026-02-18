use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use std::sync::Arc;

use crate::auth::{check_auth, check_rate_limit, extract_client_ip, record_auth_failure};
use crate::db::queries;
use crate::error::AppError;
use crate::models::request::AckRequest;
use crate::models::response::{NotificationListResponse, StatusOk};
use crate::router::AppState;

#[derive(serde::Deserialize)]
pub struct NotificationQuery {
    pub after: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list_notifications_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<NotificationListResponse>, AppError> {
    let ip = extract_client_ip(&headers);
    check_rate_limit(&state.auth_failures, ip)?;
    if let Err(e) = check_auth(&headers, &state.api_key) {
        record_auth_failure(&state.auth_failures, ip);
        return Err(e);
    }

    let limit = query.limit.unwrap_or(50).min(200);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    let notifications = queries::list_notifications(&conn, query.after.as_deref(), limit)?;

    Ok(Json(NotificationListResponse { notifications }))
}

pub async fn acknowledge_notifications_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<AckRequest>,
) -> Result<Json<StatusOk>, AppError> {
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

    queries::acknowledge_notifications(&conn, &payload.ids)?;

    Ok(Json(StatusOk::ok()))
}
