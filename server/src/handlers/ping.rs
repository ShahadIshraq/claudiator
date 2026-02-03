use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use std::sync::Arc;

use crate::auth::check_auth;
use crate::error::AppError;
use crate::models::response::StatusOk;
use crate::router::AppState;

pub async fn ping_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<StatusOk>, AppError> {
    check_auth(&headers, &state.api_key)?;
    Ok(Json(StatusOk::with_version()))
}
