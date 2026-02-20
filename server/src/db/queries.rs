#![allow(clippy::option_if_let_else)]
#![allow(clippy::missing_errors_doc)]

use rusqlite::Connection;

use crate::error::AppError;
use crate::models::response::{
    DeviceResponse, EventResponse, NotificationResponse, SessionResponse,
};

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
    .map_err(|e| AppError::Internal(format!("Failed to upsert device: {e}")))?;
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
    .map_err(|e| AppError::Internal(format!("Failed to upsert session: {e}")))?;

    // If we derived a status, update it separately (only when status is Some)
    if let Some(s) = status {
        conn.execute(
            "UPDATE sessions SET status = ?1 WHERE session_id = ?2",
            rusqlite::params![s, session_id],
        )
        .map_err(|e| AppError::Internal(format!("Failed to update session status: {e}")))?;
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
    .map_err(|e| AppError::Internal(format!("Failed to insert event: {e}")))?;
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
        .map_err(|e| AppError::Internal(format!("Failed to prepare devices query: {e}")))?;

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
        .map_err(|e| AppError::Internal(format!("Failed to query devices: {e}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect devices: {e}")))?;

    Ok(devices)
}

pub fn list_sessions(
    conn: &Connection,
    device_id: &str,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<SessionResponse>, AppError> {
    let mut sql = "SELECT s.session_id, s.device_id, s.started_at, s.last_event, s.status, s.cwd, s.title, d.device_name, d.platform
             FROM sessions s
             LEFT JOIN devices d ON d.device_id = s.device_id
             WHERE s.device_id = :device_id".to_string();

    let mut params: Vec<(&str, Box<dyn rusqlite::types::ToSql>)> =
        vec![(":device_id", Box::new(device_id.to_string()))];

    if let Some(s) = status {
        sql.push_str(" AND s.status = :status");
        params.push((":status", Box::new(s.to_string())));
    }

    sql.push_str(" ORDER BY s.last_event DESC LIMIT :limit");
    params.push((":limit", Box::new(limit)));

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::Internal(format!("Failed to prepare sessions query: {e}")))?;

    let params_refs: Vec<(&str, &dyn rusqlite::types::ToSql)> =
        params.iter().map(|(k, v)| (*k, v.as_ref())).collect();

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
        .map_err(|e| AppError::Internal(format!("Failed to query sessions: {e}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect sessions: {e}")))?;

    Ok(sessions)
}

pub fn list_all_sessions(
    conn: &Connection,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<SessionResponse>, AppError> {
    let mut sql = "SELECT s.session_id, s.device_id, s.started_at, s.last_event, s.status, s.cwd, s.title, d.device_name, d.platform
             FROM sessions s
             LEFT JOIN devices d ON d.device_id = s.device_id
             WHERE 1=1".to_string();

    let mut params: Vec<(&str, Box<dyn rusqlite::types::ToSql>)> = vec![];

    if let Some(s) = status {
        sql.push_str(" AND s.status = :status");
        params.push((":status", Box::new(s.to_string())));
    }

    sql.push_str(" ORDER BY s.last_event DESC LIMIT :limit");
    params.push((":limit", Box::new(limit)));

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::Internal(format!("Failed to prepare sessions query: {e}")))?;

    let params_refs: Vec<(&str, &dyn rusqlite::types::ToSql)> =
        params.iter().map(|(k, v)| (*k, v.as_ref())).collect();

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
        .map_err(|e| AppError::Internal(format!("Failed to query sessions: {e}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect sessions: {e}")))?;

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
        .map_err(|e| AppError::Internal(format!("Failed to prepare events query: {e}")))?;

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
        .map_err(|e| AppError::Internal(format!("Failed to query events: {e}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect events: {e}")))?;

    Ok(events)
}

pub fn get_session_title(conn: &Connection, session_id: &str) -> Result<Option<String>, AppError> {
    let mut stmt = conn
        .prepare("SELECT title FROM sessions WHERE session_id = ?1")
        .map_err(|e| AppError::Internal(format!("Failed to prepare session title query: {e}")))?;

    let mut rows = stmt
        .query(rusqlite::params![session_id])
        .map_err(|e| AppError::Internal(format!("Failed to query session title: {e}")))?;

    if let Some(row) = rows
        .next()
        .map_err(|e| AppError::Internal(format!("Failed to fetch session title row: {e}")))?
    {
        let title: Option<String> = row
            .get(0)
            .map_err(|e| AppError::Internal(format!("Failed to get session title value: {e}")))?;
        Ok(title)
    } else {
        Ok(None)
    }
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
        rusqlite::params![platform, push_token, now, i32::from(sandbox)],
    )
    .map_err(|e| AppError::Internal(format!("Failed to upsert push token: {e}")))?;
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
    .map_err(|e| AppError::Internal(format!("Failed to insert notification: {e}")))?;
    Ok(())
}

pub fn list_notifications(
    conn: &Connection,
    after_timestamp: Option<&str>,
    limit: i64,
) -> Result<Vec<NotificationResponse>, AppError> {
    let mut sql = "SELECT id, event_id, session_id, device_id, title, body, notification_type, payload_json, created_at, acknowledged
             FROM notifications
             WHERE 1=1".to_string();

    let mut params: Vec<(&str, Box<dyn rusqlite::types::ToSql>)> = vec![];

    if let Some(ts) = after_timestamp {
        sql.push_str(" AND created_at > :after_timestamp");
        params.push((":after_timestamp", Box::new(ts.to_string())));
    }

    sql.push_str(" ORDER BY created_at ASC LIMIT :limit");
    params.push((":limit", Box::new(limit)));

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AppError::Internal(format!("Failed to prepare notifications query: {e}")))?;

    let params_refs: Vec<(&str, &dyn rusqlite::types::ToSql)> =
        params.iter().map(|(k, v)| (*k, v.as_ref())).collect();

    let notifications = stmt
        .query_map(params_refs.as_slice(), |row| {
            let acknowledged_int: i32 = row.get(9)?;
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
                acknowledged: acknowledged_int != 0,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query notifications: {e}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect notifications: {e}")))?;

    Ok(notifications)
}

pub fn delete_expired_notifications(conn: &Connection) -> Result<usize, AppError> {
    let cutoff = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::hours(24))
        .ok_or_else(|| AppError::Internal("Time calculation overflow".to_string()))?
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let count = conn
        .execute(
            "DELETE FROM notifications WHERE created_at < ?1",
            rusqlite::params![cutoff],
        )
        .map_err(|e| AppError::Internal(format!("Failed to delete expired notifications: {e}")))?;

    Ok(count)
}

pub fn delete_old_events(conn: &Connection, retention_days: u64) -> Result<usize, AppError> {
    #[allow(clippy::cast_possible_wrap)]
    let cutoff = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(retention_days as i64))
        .ok_or_else(|| AppError::Internal("Time calculation overflow".to_string()))?
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let count = conn
        .execute(
            "DELETE FROM events WHERE received_at < ?1",
            rusqlite::params![cutoff],
        )
        .map_err(|e| AppError::Internal(format!("Failed to delete old events: {e}")))?;

    Ok(count)
}

pub fn delete_stale_sessions(conn: &Connection, retention_days: u64) -> Result<usize, AppError> {
    #[allow(clippy::cast_possible_wrap)]
    let cutoff = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(retention_days as i64))
        .ok_or_else(|| AppError::Internal("Time calculation overflow".to_string()))?
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let count = conn
        .execute(
            "DELETE FROM sessions WHERE last_event < ?1
               AND session_id NOT IN (SELECT DISTINCT session_id FROM events)
               AND session_id NOT IN (SELECT DISTINCT session_id FROM notifications)",
            rusqlite::params![cutoff],
        )
        .map_err(|e| AppError::Internal(format!("Failed to delete stale sessions: {e}")))?;

    Ok(count)
}

pub fn delete_stale_devices(conn: &Connection, retention_days: u64) -> Result<usize, AppError> {
    #[allow(clippy::cast_possible_wrap)]
    let cutoff = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(retention_days as i64))
        .ok_or_else(|| AppError::Internal("Time calculation overflow".to_string()))?
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let count = conn
        .execute(
            "DELETE FROM devices WHERE last_seen < ?1
               AND device_id NOT IN (SELECT DISTINCT device_id FROM sessions)
               AND device_id NOT IN (SELECT DISTINCT device_id FROM events)",
            rusqlite::params![cutoff],
        )
        .map_err(|e| AppError::Internal(format!("Failed to delete stale devices: {e}")))?;

    Ok(count)
}

pub struct PushTokenRow {
    pub push_token: String,
    #[allow(dead_code)]
    pub platform: String,
    pub sandbox: bool,
}

pub fn list_push_tokens(conn: &Connection) -> Result<Vec<PushTokenRow>, AppError> {
    let mut stmt = conn
        .prepare("SELECT push_token, platform, sandbox FROM push_tokens")
        .map_err(|e| AppError::Internal(format!("Failed to prepare push tokens query: {e}")))?;

    let tokens = stmt
        .query_map([], |row| {
            let sandbox_int: i32 = row.get(2)?;
            Ok(PushTokenRow {
                push_token: row.get(0)?,
                platform: row.get(1)?,
                sandbox: sandbox_int != 0,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query push tokens: {e}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect push tokens: {e}")))?;

    Ok(tokens)
}

pub fn delete_push_token(conn: &Connection, push_token: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM push_tokens WHERE push_token = ?1",
        rusqlite::params![push_token],
    )
    .map_err(|e| AppError::Internal(format!("Failed to delete push token: {e}")))?;
    Ok(())
}

pub fn get_metadata(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let mut stmt = conn
        .prepare("SELECT value FROM metadata WHERE key = ?1")
        .map_err(|e| AppError::Internal(format!("Failed to prepare metadata query: {e}")))?;

    let mut rows = stmt
        .query(rusqlite::params![key])
        .map_err(|e| AppError::Internal(format!("Failed to query metadata: {e}")))?;

    if let Some(row) = rows
        .next()
        .map_err(|e| AppError::Internal(format!("Failed to fetch metadata row: {e}")))?
    {
        let value: String = row
            .get(0)
            .map_err(|e| AppError::Internal(format!("Failed to get metadata value: {e}")))?;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

pub fn set_metadata(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO metadata (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )
    .map_err(|e| AppError::Internal(format!("Failed to set metadata: {e}")))?;
    Ok(())
}

pub fn acknowledge_notifications(conn: &Connection, ids: &[String]) -> Result<(), AppError> {
    if ids.is_empty() {
        return Ok(());
    }

    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("UPDATE notifications SET acknowledged = 1 WHERE id IN ({placeholders})");

    let params: Vec<&dyn rusqlite::types::ToSql> = ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();

    conn.execute(&sql, params.as_slice())
        .map_err(|e| AppError::Internal(format!("Failed to acknowledge notifications: {e}")))?;

    Ok(())
}

pub struct ApiKeyRow {
    pub id: String,
    pub name: String,
    pub key: String,
    pub scopes: String,
    pub created_at: String,
    pub last_used: Option<String>,
}

pub fn insert_api_key(
    conn: &Connection,
    id: &str,
    name: &str,
    key: &str,
    scopes: &str,
    created_at: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO api_keys (id, name, key, scopes, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, name, key, scopes, created_at],
    )
    .map_err(|e| AppError::Internal(format!("Failed to insert api key: {e}")))?;
    Ok(())
}

pub fn list_api_keys(conn: &Connection) -> Result<Vec<ApiKeyRow>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, name, key, scopes, created_at, last_used FROM api_keys ORDER BY created_at ASC")
        .map_err(|e| AppError::Internal(format!("Failed to prepare api_keys query: {e}")))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ApiKeyRow {
                id: row.get(0)?,
                name: row.get(1)?,
                key: row.get(2)?,
                scopes: row.get(3)?,
                created_at: row.get(4)?,
                last_used: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Internal(format!("Failed to query api_keys: {e}")))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Internal(format!("Failed to collect api_keys: {e}")))?;

    Ok(rows)
}

pub fn find_api_key_by_key(conn: &Connection, key: &str) -> Result<Option<ApiKeyRow>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, name, key, scopes, created_at, last_used FROM api_keys WHERE key = ?1")
        .map_err(|e| AppError::Internal(format!("Failed to prepare api_key lookup: {e}")))?;

    let mut rows = stmt
        .query(rusqlite::params![key])
        .map_err(|e| AppError::Internal(format!("Failed to query api_key: {e}")))?;

    if let Some(row) = rows
        .next()
        .map_err(|e| AppError::Internal(format!("Failed to fetch api_key row: {e}")))?
    {
        Ok(Some(ApiKeyRow {
            id: row
                .get(0)
                .map_err(|e| AppError::Internal(format!("Failed to get api_key id: {e}")))?,
            name: row
                .get(1)
                .map_err(|e| AppError::Internal(format!("Failed to get api_key name: {e}")))?,
            key: row
                .get(2)
                .map_err(|e| AppError::Internal(format!("Failed to get api_key key: {e}")))?,
            scopes: row
                .get(3)
                .map_err(|e| AppError::Internal(format!("Failed to get api_key scopes: {e}")))?,
            created_at: row.get(4).map_err(|e| {
                AppError::Internal(format!("Failed to get api_key created_at: {e}"))
            })?,
            last_used: row
                .get(5)
                .map_err(|e| AppError::Internal(format!("Failed to get api_key last_used: {e}")))?,
        }))
    } else {
        Ok(None)
    }
}

pub fn delete_api_key(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM api_keys WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| AppError::Internal(format!("Failed to delete api_key: {e}")))?;
    Ok(())
}

pub fn update_api_key_last_used(conn: &Connection, id: &str, now: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE api_keys SET last_used = ?1 WHERE id = ?2",
        rusqlite::params![now, id],
    )
    .map_err(|e| AppError::Internal(format!("Failed to update api_key last_used: {e}")))?;
    Ok(())
}
