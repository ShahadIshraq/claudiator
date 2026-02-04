use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::{SecondsFormat, Utc};
use std::sync::Arc;

use crate::auth::check_auth;
use crate::db::queries;
use crate::error::AppError;
use crate::models::request::EventPayload;
use crate::models::response::StatusOk;
use crate::router::AppState;

pub async fn events_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<EventPayload>,
) -> Result<Json<StatusOk>, AppError> {
    check_auth(&headers, &state.api_key)?;

    // Validate required fields
    if payload.device.device_id.is_empty() {
        return Err(AppError::BadRequest("device_id is required".into()));
    }
    if payload.event.session_id.is_empty() {
        return Err(AppError::BadRequest("session_id is required".into()));
    }
    if payload.event.hook_event_name.is_empty() {
        return Err(AppError::BadRequest("hook_event_name is required".into()));
    }

    // Validate timestamp is valid RFC3339
    if chrono::DateTime::parse_from_rfc3339(&payload.timestamp).is_err() {
        return Err(AppError::BadRequest("timestamp must be valid RFC 3339".into()));
    }

    let received_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    // Extract title from UserPromptSubmit events
    let title: Option<String> = if payload.event.hook_event_name == "UserPromptSubmit" {
        payload.event.prompt.as_deref().map(|p| {
            if p.len() > 200 {
                // Find a safe char boundary at or before position 200
                let mut boundary = 200;
                while boundary > 0 && !p.is_char_boundary(boundary) {
                    boundary -= 1;
                }
                format!("{}…", &p[..boundary])
            } else {
                p.to_string()
            }
        })
    } else {
        None
    };

    // Derive session status
    let session_status = derive_session_status(
        &payload.event.hook_event_name,
        payload.event.notification_type.as_deref(),
    );

    // Serialize the full event as JSON for storage
    let event_json = serde_json::to_string(&payload.event)
        .map_err(|e| AppError::Internal(format!("Failed to serialize event: {}", e)))?;

    // Get a connection from the pool
    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {}", e)))?;

    // Execute all inserts in a transaction
    conn.execute_batch("BEGIN")
        .map_err(|e| AppError::Internal(format!("Transaction begin failed: {}", e)))?;

    let result = (|| {
        queries::upsert_device(
            &conn,
            &payload.device.device_id,
            &payload.device.device_name,
            &payload.device.platform,
            &received_at,
        )?;

        queries::upsert_session(
            &conn,
            &payload.event.session_id,
            &payload.device.device_id,
            &received_at,
            session_status.as_deref(),
            payload.event.cwd.as_deref(),
            title.as_deref(),
        )?;

        queries::insert_event(
            &conn,
            &payload.device.device_id,
            &payload.event.session_id,
            &payload.event.hook_event_name,
            &payload.timestamp,
            &received_at,
            payload.event.tool_name.as_deref(),
            payload.event.notification_type.as_deref(),
            &event_json,
        )?;

        Ok::<(), AppError>(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT")
                .map_err(|e| AppError::Internal(format!("Transaction commit failed: {}", e)))?;
            state.version.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(e);
        }
    }

    tracing::info!(
        device_id = %payload.device.device_id,
        session_id = %payload.event.session_id,
        event = %payload.event.hook_event_name,
        "Event ingested"
    );

    Ok(Json(StatusOk::ok()))
}

fn derive_session_status(hook_event_name: &str, notification_type: Option<&str>) -> Option<String> {
    match hook_event_name {
        "SessionStart" | "UserPromptSubmit" => Some("active".to_string()),
        "Stop" => Some("waiting_for_input".to_string()),
        "SessionEnd" => Some("ended".to_string()),
        "Notification" => match notification_type {
            Some("permission_prompt") => Some("waiting_for_permission".to_string()),
            Some("idle_prompt") => Some("idle".to_string()),
            _ => None,
        },
        _ => None,
    }
}
