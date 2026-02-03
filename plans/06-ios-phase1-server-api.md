# Phase 1 — Server API Additions for Mobile

The server currently only has `GET /api/v1/ping` and `POST /api/v1/events`. The iOS app needs read endpoints to display data, and a registration endpoint for push tokens.

## New Endpoints

### 1. `GET /api/v1/devices`

List all known devices.

**Response:**
```json
{
  "devices": [
    {
      "device_id": "550e8400-...",
      "device_name": "shahads-macbook",
      "platform": "mac",
      "first_seen": "2026-01-15T10:00:00.000Z",
      "last_seen": "2026-02-03T14:30:00.000Z",
      "active_sessions": 1
    }
  ]
}
```

**Query:** Join `devices` with `sessions` where `status != 'ended'` to get `active_sessions` count.

### 2. `GET /api/v1/devices/:device_id/sessions`

List sessions for a device, ordered by `last_event` descending.

**Query params:**
- `status` (optional) — filter by status (`active`, `ended`, `waiting_for_input`, etc.)
- `limit` (optional, default 50)

**Response:**
```json
{
  "sessions": [
    {
      "session_id": "abc123",
      "device_id": "550e8400-...",
      "started_at": "2026-02-03T14:00:00.000Z",
      "last_event": "2026-02-03T14:30:00.000Z",
      "status": "waiting_for_input",
      "cwd": "/Users/shahad/workspace/project"
    }
  ]
}
```

### 3. `GET /api/v1/sessions/:session_id/events`

List events for a session, ordered by `timestamp` descending.

**Query params:**
- `limit` (optional, default 100)

**Response:**
```json
{
  "events": [
    {
      "id": 42,
      "hook_event_name": "Stop",
      "timestamp": "2026-02-03T14:30:00.000Z",
      "tool_name": null,
      "notification_type": null,
      "message": null
    }
  ]
}
```

**Note:** Return selected fields from the event, not the full `event_json` blob. Keep payloads small for mobile.

### 4. `POST /api/v1/push/register`

Register a mobile device's push token.

**Request:**
```json
{
  "platform": "ios",
  "push_token": "a1b2c3d4e5..."
}
```

**Response:**
```json
{
  "status": "ok"
}
```

**Storage:** New `push_tokens` table (separate from `devices` — these are mobile app instances, not Claude Code machines).

## Database Changes

### New table: `push_tokens`

```sql
CREATE TABLE push_tokens (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    platform    TEXT NOT NULL,           -- "ios" or "android"
    push_token  TEXT NOT NULL UNIQUE,    -- APNs or FCM token
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX idx_push_tokens_platform ON push_tokens(platform);
```

This table is intentionally simple — it stores mobile app push tokens associated with the API key. Since auth is a single shared API key, all registered tokens receive all notifications.

## Implementation Notes

- All new endpoints require Bearer auth (same `check_auth` pattern)
- Query endpoints are read-only — use `conn.prepare` + `query_map`
- Keep response structs in `models/response.rs`
- Add new handler files: `handlers/devices.rs`, `handlers/sessions.rs`, `handlers/push.rs`
- Add routes in `router.rs`
- Run migration for `push_tokens` table in `db/migrations.rs`
