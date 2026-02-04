use rusqlite::Connection;

use crate::error::AppError;
use crate::models::response::{DeviceResponse, EventResponse, NotificationResponse, SessionResponse};

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
) -> Result<i64, AppError> {
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
    Ok(conn.last_insert_rowid())
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
            "SELECT s.session_id, s.device_id, s.started_at, s.last_event, s.status, s.cwd, s.title, d.device_name, d.platform
             FROM sessions s
             LEFT JOIN devices d ON d.device_id = s.device_id
             WHERE s.device_id = ?1 AND s.status = ?2
             ORDER BY s.last_event DESC
             LIMIT ?3"
                .to_string(),
            vec![
                Box::new(device_id.to_string()),
                Box::new(s.to_string()),
                Box::new(limit),
            ],
        ),
        None => (
            "SELECT s.session_id, s.device_id, s.started_at, s.last_event, s.status, s.cwd, s.title, d.device_name, d.platform
             FROM sessions s
             LEFT JOIN devices d ON d.device_id = s.device_id
             WHERE s.device_id = ?1
             ORDER BY s.last_event DESC
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
                device_name: row.get(7)?,
                platform: row.get(8)?,
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
            "SELECT s.session_id, s.device_id, s.started_at, s.last_event, s.status, s.cwd, s.title, d.device_name, d.platform
             FROM sessions s
             LEFT JOIN devices d ON d.device_id = s.device_id
             WHERE s.status = ?1
             ORDER BY s.last_event DESC
             LIMIT ?2"
                .to_string(),
            vec![Box::new(s.to_string()), Box::new(limit)],
        ),
        None => (
            "SELECT s.session_id, s.device_id, s.started_at, s.last_event, s.status, s.cwd, s.title, d.device_name, d.platform
             FROM sessions s
             LEFT JOIN devices d ON d.device_id = s.device_id
             ORDER BY s.last_event DESC
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
                device_name: row.get(7)?,
                platform: row.get(8)?,
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
    sandbox: bool,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO push_tokens (platform, push_token, created_at, updated_at, sandbox)
         VALUES (?1, ?2, ?3, ?3, ?4)
         ON CONFLICT(push_token) DO UPDATE SET
            platform = excluded.platform,
            updated_at = excluded.updated_at,
            sandbox = excluded.sandbox",
        rusqlite::params![platform, push_token, now, sandbox as i32],
    )
    .map_err(|e| AppError::Internal(format!("Failed to upsert push token: {}", e)))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn insert_notification(
    conn: &Connection,
    id: &str,
    event_id: i64,
    session_id: &str,
    device_id: &str,
    title: &str,
    body: &str,
    notification_type: &str,
    payload_json: Option<&str>,
    created_at: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO notifications (id, event_id, session_id, device_id, title, body, notification_type, payload_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, event_id, session_id, device_id, title, body, notification_type, payload_json, created_at],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert notification: {}", e)))?;
    Ok(())
}

pub fn list_notifications(
    conn: &Connection,
    since_id: Option<&str>,
    limit: i64,
) -> Result<Vec<NotificationResponse>, AppError> {
    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match since_id {
        Some(id) => (
            "SELECT id, event_id, session_id, device_id, title, body, notification_type, payload_json, created_at
             FROM notifications
             WHERE created_at > (SELECT created_at FROM notifications WHERE id = ?1)
             ORDER BY created_at ASC
             LIMIT ?2".to_string(),
            vec![Box::new(id.to_string()), Box::new(limit)],
        ),
        None => (
            "SELECT id, event_id, session_id, device_id, title, body, notification_type, payload_json, created_at
             FROM notifications
             ORDER BY created_at ASC
             LIMIT ?1".to_string(),
            vec![Box::new(limit)],
        ),
    };

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::Internal(format!("Failed to prepare notifications query: {}", e)))?;

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let notifications = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(NotificationResponse {
                id: row.get(0)?,
                event_id: row.get(1)?,
                session_id: row.get(2)?,
                device_id: row.get(3)?,
                title: row.get(4)?,
                body: row.get(5)?,
                notification_type: row.get(6)?,
                payload_json: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query notifications: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect notifications: {}", e)))?;

    Ok(notifications)
}

pub fn delete_expired_notifications(conn: &Connection) -> Result<usize, AppError> {
    let cutoff = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::hours(24))
        .unwrap()
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let count = conn
        .execute(
            "DELETE FROM notifications WHERE created_at < ?1",
            rusqlite::params![cutoff],
        )
        .map_err(|e| AppError::Internal(format!("Failed to delete expired notifications: {}", e)))?;

    Ok(count)
}

pub struct PushTokenRow {
    pub push_token: String,
    pub platform: String,
    pub sandbox: bool,
}

pub fn list_push_tokens(conn: &Connection) -> Result<Vec<PushTokenRow>, AppError> {
    let mut stmt = conn
        .prepare("SELECT push_token, platform, sandbox FROM push_tokens")
        .map_err(|e| AppError::Internal(format!("Failed to prepare push tokens query: {}", e)))?;

    let tokens = stmt
        .query_map([], |row| {
            let sandbox_int: i32 = row.get(2)?;
            Ok(PushTokenRow {
                push_token: row.get(0)?,
                platform: row.get(1)?,
                sandbox: sandbox_int != 0,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query push tokens: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect push tokens: {}", e)))?;

    Ok(tokens)
}

pub fn delete_push_token(conn: &Connection, push_token: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM push_tokens WHERE push_token = ?1",
        rusqlite::params![push_token],
    )
    .map_err(|e| AppError::Internal(format!("Failed to delete push token: {}", e)))?;
    Ok(())
}
