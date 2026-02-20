use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{SecondsFormat, Utc};
use std::sync::Arc;

use crate::auth::AdminAuth;
use crate::db::queries;
use crate::error::AppError;
use crate::models::request::CreateApiKeyRequest;
use crate::models::response::{
    ApiKeyCreatedResponse, ApiKeyListItem, ApiKeyListResponse, StatusOk,
};
use crate::router::AppState;

fn generate_api_key() -> String {
    format!("claud_{}", uuid::Uuid::new_v4().simple())
}

pub async fn create_api_key_handler(
    State(state): State<Arc<AppState>>,
    _auth: AdminAuth,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<ApiKeyCreatedResponse>), AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    if payload.scopes.is_empty() {
        return Err(AppError::BadRequest("scopes must not be empty".into()));
    }

    // Validate and deduplicate scopes
    let mut validated: Vec<String> = Vec::new();
    for s in &payload.scopes {
        match s.as_str() {
            "read" | "write" => {
                if !validated.contains(s) {
                    validated.push(s.clone());
                }
            }
            other => {
                return Err(AppError::BadRequest(format!(
                    "invalid scope '{}': must be 'read' or 'write'",
                    other
                )));
            }
        }
    }

    let id = uuid::Uuid::new_v4().to_string();
    let key = generate_api_key();
    let scopes_str = validated.join(",");
    let created_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    queries::insert_api_key(
        &conn,
        &id,
        payload.name.trim(),
        &key,
        &scopes_str,
        &created_at,
    )?;

    tracing::info!(name = %payload.name.trim(), scopes = %scopes_str, "API key created");

    Ok((
        StatusCode::CREATED,
        Json(ApiKeyCreatedResponse {
            id,
            name: payload.name.trim().to_string(),
            key,
            scopes: validated,
            created_at,
        }),
    ))
}

pub async fn list_api_keys_handler(
    State(state): State<Arc<AppState>>,
    _auth: AdminAuth,
) -> Result<Json<ApiKeyListResponse>, AppError> {
    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    let rows = queries::list_api_keys(&conn)?;

    let keys = rows
        .into_iter()
        .map(|row| {
            let key_prefix = row.key.chars().take(12).collect::<String>();
            let scopes = row
                .scopes
                .split(',')
                .map(str::to_string)
                .collect::<Vec<_>>();
            ApiKeyListItem {
                id: row.id,
                name: row.name,
                key_prefix,
                scopes,
                created_at: row.created_at,
                last_used: row.last_used,
            }
        })
        .collect();

    Ok(Json(ApiKeyListResponse { keys }))
}

pub async fn delete_api_key_handler(
    State(state): State<Arc<AppState>>,
    _auth: AdminAuth,
    Path(id): Path<String>,
) -> Result<Json<StatusOk>, AppError> {
    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    queries::delete_api_key(&conn, &id)?;

    tracing::info!(id = %id, "API key deleted");

    Ok(Json(StatusOk::ok()))
}
