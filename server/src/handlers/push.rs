use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::{SecondsFormat, Utc};
use std::sync::Arc;

use crate::auth::{check_auth, check_rate_limit, extract_client_ip, record_auth_failure};
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
    let ip = extract_client_ip(&headers);
    check_rate_limit(&state.auth_failures, ip)?;
    if let Err(e) = check_auth(&headers, &state.api_key) {
        record_auth_failure(&state.auth_failures, ip);
        return Err(e);
    }

    if payload.platform.is_empty() {
        return Err(AppError::BadRequest("platform is required".into()));
    }
    if payload.push_token.is_empty() {
        return Err(AppError::BadRequest("push_token is required".into()));
    }

    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let sandbox = payload.sandbox.unwrap_or(false);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    queries::upsert_push_token(&conn, &payload.platform, &payload.push_token, &now, sandbox)?;

    tracing::info!(
        platform = %payload.platform,
        "Push token registered"
    );

    Ok(Json(StatusOk::ok()))
}
