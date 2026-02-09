use crate::db::pool::DbPool;

pub fn run(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = pool.get()?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS devices (
            device_id   TEXT PRIMARY KEY,
            device_name TEXT NOT NULL,
            platform    TEXT NOT NULL,
            first_seen  TEXT NOT NULL,
            last_seen   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sessions (
            session_id  TEXT PRIMARY KEY,
            device_id   TEXT NOT NULL REFERENCES devices(device_id),
            started_at  TEXT NOT NULL,
            last_event  TEXT NOT NULL,
            status      TEXT NOT NULL DEFAULT 'active',
            cwd         TEXT
        );

        CREATE TABLE IF NOT EXISTS events (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id         TEXT NOT NULL,
            session_id        TEXT NOT NULL,
            hook_event_name   TEXT NOT NULL,
            timestamp         TEXT NOT NULL,
            received_at       TEXT NOT NULL,
            tool_name         TEXT,
            notification_type TEXT,
            event_json        TEXT NOT NULL,
            FOREIGN KEY (device_id) REFERENCES devices(device_id),
            FOREIGN KEY (session_id) REFERENCES sessions(session_id)
        );

        CREATE INDEX IF NOT EXISTS idx_events_session_id ON events(session_id);
        CREATE INDEX IF NOT EXISTS idx_events_device_id ON events(device_id);
        CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
        CREATE INDEX IF NOT EXISTS idx_events_hook_event_name ON events(hook_event_name);
        CREATE INDEX IF NOT EXISTS idx_sessions_device_id ON sessions(device_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);

        CREATE TABLE IF NOT EXISTS push_tokens (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            platform    TEXT NOT NULL,
            push_token  TEXT NOT NULL UNIQUE,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_push_tokens_platform ON push_tokens(platform);",
    )?;

    // Add title column to sessions (idempotent)
    let _ = conn.execute("ALTER TABLE sessions ADD COLUMN title TEXT", []);

    // Add notifications table (idempotent)
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS notifications (
            id                TEXT PRIMARY KEY,
            event_id          INTEGER NOT NULL,
            session_id        TEXT NOT NULL,
            device_id         TEXT NOT NULL,
            title             TEXT NOT NULL,
            body              TEXT NOT NULL,
            notification_type TEXT NOT NULL,
            payload_json      TEXT,
            created_at        TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES sessions(session_id)
        );

        CREATE INDEX IF NOT EXISTS idx_notifications_session_id ON notifications(session_id);
        CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at);",
    )?;

    // Add sandbox column to push_tokens (idempotent)
    let _ = conn.execute(
        "ALTER TABLE push_tokens ADD COLUMN sandbox INTEGER NOT NULL DEFAULT 0",
        [],
    );

    tracing::info!("Database migrations complete");
    Ok(())
}
