use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use std::sync::Arc;

use crate::auth::check_auth;
use crate::db::queries;
use crate::error::AppError;
use crate::models::response::NotificationListResponse;
use crate::router::AppState;

#[derive(serde::Deserialize)]
pub struct NotificationQuery {
    pub since: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list_notifications_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<NotificationListResponse>, AppError> {
    check_auth(&headers, &state.api_key)?;

    let limit = query.limit.unwrap_or(50).min(200);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {}", e)))?;

    let notifications = queries::list_notifications(&conn, query.since.as_deref(), limit)?;

    Ok(Json(NotificationListResponse { notifications }))
}
