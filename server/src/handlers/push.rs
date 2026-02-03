use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::{SecondsFormat, Utc};
use std::sync::Arc;

use crate::auth::check_auth;
use crate::db::queries;
use crate::error::AppError;
use crate::models::request::PushRegisterRequest;
use crate::models::response::StatusOk;
use crate::router::AppState;

pub async fn push_register_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<PushRegisterRequest>,
) -> Result<Json<StatusOk>, AppError> {
    check_auth(&headers, &state.api_key)?;

    if payload.platform.is_empty() {
        return Err(AppError::BadRequest("platform is required".into()));
    }
    if payload.push_token.is_empty() {
        return Err(AppError::BadRequest("push_token is required".into()));
    }

    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {}", e)))?;

    queries::upsert_push_token(&conn, &payload.platform, &payload.push_token, &now)?;

    tracing::info!(
        platform = %payload.platform,
        "Push token registered"
    );

    Ok(Json(StatusOk::ok()))
}
