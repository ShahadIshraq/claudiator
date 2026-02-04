use rusqlite::Connection;

use crate::error::AppError;
use crate::models::response::{DeviceResponse, EventResponse, SessionResponse};

pub fn upsert_device(
    conn: &Connection,
    device_id: &str,
    device_name: &str,
    platform: &str,
    now: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO devices (device_id, device_name, platform, first_seen, last_seen)
         VALUES (?1, ?2, ?3, ?4, ?4)
         ON CONFLICT(device_id) DO UPDATE SET
            device_name = excluded.device_name,
            last_seen = excluded.last_seen",
        rusqlite::params![device_id, device_name, platform, now],
    )
    .map_err(|e| AppError::Internal(format!("Failed to upsert device: {}", e)))?;
    Ok(())
}

pub fn upsert_session(
    conn: &Connection,
    session_id: &str,
    device_id: &str,
    now: &str,
    status: Option<&str>,
    cwd: Option<&str>,
    title: Option<&str>,
) -> Result<(), AppError> {
    // First try to insert. If it conflicts, update selectively.
    let initial_status = status.unwrap_or("active");

    conn.execute(
        "INSERT INTO sessions (session_id, device_id, started_at, last_event, status, cwd, title)
         VALUES (?1, ?2, ?3, ?3, ?4, ?5, ?6)
         ON CONFLICT(session_id) DO UPDATE SET
            last_event = excluded.last_event,
            cwd = COALESCE(excluded.cwd, sessions.cwd),
            title = COALESCE(sessions.title, excluded.title)",
        rusqlite::params![session_id, device_id, now, initial_status, cwd, title],
    )
    .map_err(|e| AppError::Internal(format!("Failed to upsert session: {}", e)))?;

    // If we derived a status, update it separately (only when status is Some)
    if let Some(s) = status {
        conn.execute(
            "UPDATE sessions SET status = ?1 WHERE session_id = ?2",
            rusqlite::params![s, session_id],
        )
        .map_err(|e| AppError::Internal(format!("Failed to update session status: {}", e)))?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn insert_event(
    conn: &Connection,
    device_id: &str,
    session_id: &str,
    hook_event_name: &str,
    timestamp: &str,
    received_at: &str,
    tool_name: Option<&str>,
    notification_type: Option<&str>,
    event_json: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO events (device_id, session_id, hook_event_name, timestamp, received_at, tool_name, notification_type, event_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            device_id,
            session_id,
            hook_event_name,
            timestamp,
            received_at,
            tool_name,
            notification_type,
            event_json,
        ],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert event: {}", e)))?;
    Ok(())
}

pub fn list_devices(conn: &Connection) -> Result<Vec<DeviceResponse>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT d.device_id, d.device_name, d.platform, d.first_seen, d.last_seen,
                    (SELECT COUNT(*) FROM sessions s WHERE s.device_id = d.device_id AND s.status != 'ended') AS active_sessions
             FROM devices d
             ORDER BY d.last_seen DESC",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare devices query: {}", e)))?;

    let devices = stmt
        .query_map([], |row| {
            Ok(DeviceResponse {
                device_id: row.get(0)?,
                device_name: row.get(1)?,
                platform: row.get(2)?,
                first_seen: row.get(3)?,
                last_seen: row.get(4)?,
                active_sessions: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query devices: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect devices: {}", e)))?;

    Ok(devices)
}

pub fn list_sessions(
    conn: &Connection,
    device_id: &str,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<SessionResponse>, AppError> {
    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match status {
        Some(s) => (
            "SELECT session_id, device_id, started_at, last_event, status, cwd, title
             FROM sessions
             WHERE device_id = ?1 AND status = ?2
             ORDER BY last_event DESC
             LIMIT ?3"
                .to_string(),
            vec![
                Box::new(device_id.to_string()),
                Box::new(s.to_string()),
                Box::new(limit),
            ],
        ),
        None => (
            "SELECT session_id, device_id, started_at, last_event, status, cwd, title
             FROM sessions
             WHERE device_id = ?1
             ORDER BY last_event DESC
             LIMIT ?2"
                .to_string(),
            vec![Box::new(device_id.to_string()), Box::new(limit)],
        ),
    };

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::Internal(format!("Failed to prepare sessions query: {}", e)))?;

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let sessions = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(SessionResponse {
                session_id: row.get(0)?,
                device_id: row.get(1)?,
                started_at: row.get(2)?,
                last_event: row.get(3)?,
                status: row.get(4)?,
                cwd: row.get(5)?,
                title: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query sessions: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect sessions: {}", e)))?;

    Ok(sessions)
}

pub fn list_all_sessions(
    conn: &Connection,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<SessionResponse>, AppError> {
    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match status {
        Some(s) => (
            "SELECT session_id, device_id, started_at, last_event, status, cwd, title
             FROM sessions
             WHERE status = ?1
             ORDER BY last_event DESC
             LIMIT ?2"
                .to_string(),
            vec![Box::new(s.to_string()), Box::new(limit)],
        ),
        None => (
            "SELECT session_id, device_id, started_at, last_event, status, cwd, title
             FROM sessions
             ORDER BY last_event DESC
             LIMIT ?1"
                .to_string(),
            vec![Box::new(limit)],
        ),
    };

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::Internal(format!("Failed to prepare sessions query: {}", e)))?;

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let sessions = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(SessionResponse {
                session_id: row.get(0)?,
                device_id: row.get(1)?,
                started_at: row.get(2)?,
                last_event: row.get(3)?,
                status: row.get(4)?,
                cwd: row.get(5)?,
                title: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query sessions: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect sessions: {}", e)))?;

    Ok(sessions)
}

pub fn list_events(
    conn: &Connection,
    session_id: &str,
    limit: i64,
) -> Result<Vec<EventResponse>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.hook_event_name, e.timestamp, e.tool_name, e.notification_type,
                    json_extract(e.event_json, '$.message') AS message
             FROM events e
             WHERE e.session_id = ?1
             ORDER BY e.timestamp DESC
             LIMIT ?2",
        )
        .map_err(|e| AppError::Internal(format!("Failed to prepare events query: {}", e)))?;

    let events = stmt
        .query_map(rusqlite::params![session_id, limit], |row| {
            Ok(EventResponse {
                id: row.get(0)?,
                hook_event_name: row.get(1)?,
                timestamp: row.get(2)?,
                tool_name: row.get(3)?,
                notification_type: row.get(4)?,
                message: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query events: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect events: {}", e)))?;

    Ok(events)
}

pub fn upsert_push_token(
    conn: &Connection,
    platform: &str,
    push_token: &str,
    now: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO push_tokens (platform, push_token, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?3)
         ON CONFLICT(push_token) DO UPDATE SET
            platform = excluded.platform,
            updated_at = excluded.updated_at",
        rusqlite::params![platform, push_token, now],
    )
    .map_err(|e| AppError::Internal(format!("Failed to upsert push token: {}", e)))?;
    Ok(())
}
