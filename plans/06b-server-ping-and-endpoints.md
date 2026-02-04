# Phase 4b — Server: Ping Notification Version & Notification Endpoints

## Overview

Extend the ping response with `notification_version` so clients can efficiently detect new notifications. Add endpoints for listing and acknowledging notifications.

## Ping Response Extension

`server/src/models/response.rs` — add field to `StatusOk`:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub notification_version: Option<u64>,
```

Update constructors: `ok()` and `with_version()` set it to `None`. New constructor:

```rust
pub fn with_versions(data_v: u64, notif_v: u64) -> Self
```

`server/src/handlers/ping.rs` — load both counters, return via `with_versions()`.

Response becomes:

```json
{
  "status": "ok",
  "server_version": "0.1.0",
  "data_version": 42,
  "notification_version": 7
}
```

## New Endpoints

New file: `server/src/handlers/notifications.rs`

### GET /api/v1/notifications

Query params: `since` (UUID, optional), `limit` (int, default 50)

- If `since` provided: return notifications with `created_at` > the `since` notification's `created_at`, ordered ASC
- If no `since`: return most recent N, ordered DESC
- Response: `{ "notifications": [...] }`

### POST /api/v1/notifications/ack

- Body: `{ "notification_ids": ["uuid1", "uuid2"] }`
- Sets `acknowledged = 1` for matching IDs
- Response: `{ "status": "ok" }`

## New Models

`server/src/models/response.rs`:

```rust
#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub session_id: String,
    pub device_id: String,
    pub title: String,
    pub body: String,
    pub notification_type: String,
    pub payload_json: String,
    pub created_at: String,
    pub acknowledged: bool,
}

#[derive(Debug, Serialize)]
pub struct NotificationListResponse {
    pub notifications: Vec<NotificationResponse>,
}
```

`server/src/models/request.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct NotificationAckRequest {
    pub notification_ids: Vec<String>,
}
```

## New Queries

`server/src/db/queries.rs`:
- `list_notifications(conn, since_id: Option<&str>, limit: i64) -> Vec<NotificationResponse>`
- `acknowledge_notifications(conn, ids: &[String]) -> u64`

## Route Registration

`server/src/router.rs`:

```rust
.route("/api/v1/notifications", get(handlers::notifications::list_notifications_handler))
.route("/api/v1/notifications/ack", post(handlers::notifications::ack_notifications_handler))
```

`server/src/handlers/mod.rs` — add `pub mod notifications;`

## Files Modified

| File | Action |
|---|---|
| `server/src/models/response.rs` | modify — StatusOk notification_version + NotificationResponse + NotificationListResponse |
| `server/src/models/request.rs` | modify — NotificationAckRequest |
| `server/src/handlers/ping.rs` | modify — return notification_version |
| `server/src/handlers/notifications.rs` | **create** — list + ack handlers |
| `server/src/handlers/mod.rs` | modify — pub mod notifications |
| `server/src/router.rs` | modify — add 2 routes |
| `server/src/db/queries.rs` | modify — list_notifications + acknowledge_notifications |
