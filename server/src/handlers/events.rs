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

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
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
        return Err(AppError::BadRequest(
            "timestamp must be valid RFC 3339".into(),
        ));
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
        .map_err(|e| AppError::Internal(format!("Failed to serialize event: {e}")))?;

    // Get a connection from the pool
    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("Database pool error: {e}")))?;

    // Execute all inserts in a transaction
    conn.execute_batch("BEGIN")
        .map_err(|e| AppError::Internal(format!("Transaction begin failed: {e}")))?;

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

        let event_id = queries::insert_event(
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

        Ok::<i64, AppError>(event_id)
    })();

    let event_id = match result {
        Ok(event_id) => {
            // Persist data version bump
            let new_version = state
                .version
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                + 1;
            queries::set_metadata(&conn, "data_version", &new_version.to_string())?;

            conn.execute_batch("COMMIT")
                .map_err(|e| AppError::Internal(format!("Transaction commit failed: {e}")))?;
            event_id
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(e);
        }
    };

    // Notification pipeline — after successful commit
    if let Some((notif_title, notif_body, notif_type)) = should_notify(
        &payload.event.hook_event_name,
        payload.event.notification_type.as_deref(),
        payload.event.message.as_deref(),
    ) {
        let notification_id = uuid::Uuid::new_v4().to_string();

        let _ = queries::insert_notification(
            &conn,
            &notification_id,
            event_id,
            &payload.event.session_id,
            &payload.device.device_id,
            &notif_title,
            &notif_body,
            &notif_type,
            None,
            &received_at,
        );

        // Persist notification version bump
        let new_notif_version = state
            .notification_version
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        let _ = queries::set_metadata(
            &conn,
            "notification_version",
            &new_notif_version.to_string(),
        );

        // Synchronous cleanup with time guard (max once per 5 minutes)
        #[allow(clippy::cast_sign_loss)]
        let now_secs = Utc::now().timestamp() as u64;
        let last_cleanup = state
            .last_cleanup
            .load(std::sync::atomic::Ordering::Relaxed);
        let five_minutes_secs = 5 * 60;

        if now_secs.saturating_sub(last_cleanup) >= five_minutes_secs {
            match queries::delete_expired_notifications(&conn) {
                Ok(count) if count > 0 => {
                    tracing::debug!("Cleaned up {} expired notifications", count);
                    state
                        .last_cleanup
                        .store(now_secs, std::sync::atomic::Ordering::Relaxed);
                }
                Ok(_) => {
                    state
                        .last_cleanup
                        .store(now_secs, std::sync::atomic::Ordering::Relaxed);
                }
                Err(e) => {
                    tracing::warn!("Failed to clean expired notifications: {:?}", e);
                }
            }
        }

        // APNs push dispatch
        if let Some(ref apns_client) = state.apns_client {
            let apns = apns_client.clone();
            let push_pool = state.db_pool.clone();
            let push_title = notif_title;
            let push_body = notif_body;

            // Use session_id as collapse_id with 64-byte truncation guard
            let session_id_str = &payload.event.session_id;
            let collapse_id = if session_id_str.len() > 64 {
                let mut boundary = 64;
                while boundary > 0 && !session_id_str.is_char_boundary(boundary) {
                    boundary -= 1;
                }
                session_id_str[..boundary].to_string()
            } else {
                session_id_str.clone()
            };

            let push_notification_id = notification_id;
            let push_session_id = payload.event.session_id.clone();
            let push_device_id = payload.device.device_id.clone();

            tokio::spawn(async move {
                let tokens = match push_pool.get() {
                    Ok(c) => match queries::list_push_tokens(&c) {
                        Ok(t) => t,
                        Err(e) => {
                            tracing::warn!("Failed to list push tokens: {:?}", e);
                            return;
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to get db connection for push: {}", e);
                        return;
                    }
                };

                for token_row in &tokens {
                    let result = apns
                        .send_push(
                            &token_row.push_token,
                            &push_title,
                            &push_body,
                            Some(&collapse_id),
                            &push_notification_id,
                            &push_session_id,
                            &push_device_id,
                            token_row.sandbox,
                        )
                        .await;

                    match result {
                        crate::apns::ApnsPushResult::Success => {
                            tracing::debug!(
                                "Push sent to token {}",
                                &token_row.push_token[..8.min(token_row.push_token.len())]
                            );
                        }
                        crate::apns::ApnsPushResult::Gone => {
                            tracing::info!(
                                "Push token gone, removing: {}",
                                &token_row.push_token[..8.min(token_row.push_token.len())]
                            );
                            if let Ok(c) = push_pool.get() {
                                let _ = queries::delete_push_token(&c, &token_row.push_token);
                            }
                        }
                        crate::apns::ApnsPushResult::AuthError => {
                            tracing::error!("APNs auth error — check credentials");
                        }
                        crate::apns::ApnsPushResult::Retry => {
                            tracing::warn!("APNs rate limited, skipping remaining tokens");
                            break;
                        }
                        crate::apns::ApnsPushResult::OtherError(e) => {
                            tracing::warn!("APNs push error: {}", e);
                        }
                    }
                }
            });
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
        "SessionStart" | "UserPromptSubmit" | "SubagentStart" | "SubagentStop" => {
            Some("active".to_string())
        }
        "Stop" => Some("waiting_for_input".to_string()),
        "SessionEnd" => Some("ended".to_string()),
        "PermissionRequest" => Some("waiting_for_permission".to_string()),
        "Notification" => match notification_type {
            Some("permission_prompt") => Some("waiting_for_permission".to_string()),
            Some("idle_prompt") => Some("idle".to_string()),
            _ => None,
        },
        _ => None,
    }
}

fn should_notify(
    hook_event_name: &str,
    notification_type: Option<&str>,
    message: Option<&str>,
) -> Option<(String, String, String)> {
    match hook_event_name {
        "Stop" => {
            let body = message.unwrap_or("Session has stopped").to_string();
            Some(("Session Stopped".to_string(), body, "stop".to_string()))
        }
        "Notification" => match notification_type {
            Some("permission_prompt") => Some((
                "Permission Required".to_string(),
                message
                    .unwrap_or("A session needs permission to continue")
                    .to_string(),
                "permission_prompt".to_string(),
            )),
            Some("idle_prompt") => Some((
                "Session Idle".to_string(),
                message
                    .unwrap_or("A session is waiting for input")
                    .to_string(),
                "idle_prompt".to_string(),
            )),
            _ => None,
        },
        "PermissionRequest" => Some((
            "Permission Required".to_string(),
            message.unwrap_or("A session needs permission to continue").to_string(),
            "permission_prompt".to_string(),
        )),
        _ => None,
    }
}
