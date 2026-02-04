# Phase 4a — Server: Notification Table & Generation

## Overview

Add a `notifications` table to the server database and generate notification records when notifiable events arrive. This is the foundation for all notification delivery paths.

## What Is a Notification?

Add `should_notify()` to `server/src/handlers/events.rs`:

```
fn should_notify(hook_event_name, notification_type) -> Option<NotifType>:
    "Stop"                                    → Some(stop)
    "Notification" + "permission_prompt"      → Some(permission_prompt)
    "Notification" + "idle_prompt"            → Some(idle_prompt)
    everything else                           → None
```

Content rendering:

| notification_type | Title | Body |
|---|---|---|
| stop | "Claude is waiting" | "Session idle on {device_name}" |
| permission_prompt | "Permission needed" | "{tool_name} on {device_name}" |
| idle_prompt | "Session idle" | "Idle on {device_name}" |

## Schema Migration

`server/src/db/migrations.rs` — new table:

```sql
CREATE TABLE IF NOT EXISTS notifications (
    id                TEXT PRIMARY KEY,         -- UUID v4, dedup key
    event_id          INTEGER NOT NULL REFERENCES events(id),
    session_id        TEXT NOT NULL REFERENCES sessions(session_id),
    device_id         TEXT NOT NULL REFERENCES devices(device_id),
    title             TEXT NOT NULL,
    body              TEXT NOT NULL,
    notification_type TEXT NOT NULL,
    payload_json      TEXT NOT NULL,
    acknowledged      INTEGER NOT NULL DEFAULT 0,
    created_at        TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at);
CREATE INDEX IF NOT EXISTS idx_notifications_session_id ON notifications(session_id);
```

## Event Handler Changes

`server/src/handlers/events.rs` — inside the transaction closure, after `insert_event`:

1. Call `should_notify()` with event name + notification_type
2. If notifiable: generate UUID, render title/body, build payload_json (session_id, device_id, device_name)
3. Call `queries::insert_notification()` — **inside the same transaction** so notification + event are atomic
4. After COMMIT: increment `notification_version` counter
5. After COMMIT: if APNs is configured, spawn async push dispatch (phase 4c)

## Query Changes

`server/src/db/queries.rs`:
- `insert_event()` → change return from `Result<()>` to `Result<i64>` (return `conn.last_insert_rowid()`)
- New: `insert_notification(conn, id, event_id, session_id, device_id, title, body, notification_type, payload_json, created_at)`

## AppState Change

`server/src/router.rs` — add `notification_version: AtomicU64` to `AppState`

`server/src/main.rs` — initialize `notification_version: AtomicU64::new(0)`

## New Dependency

`server/Cargo.toml` — add `uuid = { version = "1", features = ["v4"] }`

## Files Modified

| File | Action |
|---|---|
| `server/Cargo.toml` | modify — add uuid |
| `server/src/router.rs` | modify — add notification_version to AppState |
| `server/src/main.rs` | modify — init notification_version |
| `server/src/db/migrations.rs` | modify — add notifications table |
| `server/src/db/queries.rs` | modify — insert_event return type + insert_notification |
| `server/src/handlers/events.rs` | modify — should_notify + notification generation |
