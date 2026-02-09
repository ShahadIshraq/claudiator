# Claudiator — Architecture

## Components

```
+------------------+         +---------------------+         +------------------+
|   Claude Code    |         |  claudiator-hook    |         |   Claudiator     |
|   (IDE/CLI)      |         |  (Rust CLI binary)  |         |   Server         |
|                  |         |                     |         |  (Axum + SQLite) |
|  Fires hook      | stdin   |  Reads JSON event   |  HTTP   |  Stores events   |
|  events on:      |-------->|  Loads config       |-------->|  Manages devices |
|  - SessionStart  |  JSON   |  Builds payload     |  POST   |  Serves API      |
|  - SessionEnd    |         |  POSTs to server    |         |  Sends APNs push |
|  - Stop          |         |                     |         |                  |
|  - Notification  |         |  Always exits 0     |         |                  |
|  - PromptSubmit  |         |  Errors -> log file |         |                  |
+------------------+         +---------------------+         +------------------+
                                      |                               |
                                      |                               |
                                      v                               v
                             ~/.claude/claudiator/          /opt/claudiator/
                             ├── config.toml                ├── claudiator-server
                             ├── claudiator-hook            ├── claudiator.db (SQLite)
                             └── error.log                  └── .env
                                                                      |
                                                                      |
                                                   +------------------+------------------+
                                                   |                                     |
                                                   v                                     v
                                         +--------------------+             +-------------------------+
                                         |   Mobile Apps      |             |  APNs (Apple)           |
                                         |                    |    push     |  api.push.apple.com     |
                                         |  iOS (SwiftUI) ✅  |<------------|  api.sandbox.push.apple |
                                         |  Android (Planned) |             |                         |
                                         |                    |    poll     |  HTTP/2 + ES256 JWT     |
                                         |  Live session      |  (fallback) |                         |
                                         |  status, themes,   |             +-------------------------+
                                         |  hybrid notifs     |
                                         +--------------------+
```

## Data Flow

1. **Claude Code** fires a hook event (e.g. Notification) and pipes JSON to stdin
2. **claudiator-hook** reads stdin, parses the event, loads device config from `config.toml`
3. **claudiator-hook** wraps the event in a payload with device info + timestamp
4. **claudiator-hook** POSTs to the server at `POST /api/v1/events` with `Authorization: Bearer {api_key}`
5. **claudiator-server** validates the API key, stores the event in SQLite (devices, sessions, events tables)
6. **claudiator-server** generates a notification record (UUID) for Stop/permission_prompt/idle_prompt events, increments `notification_version`
7. **claudiator-server** (if APNs is configured) dispatches push notification with custom payload (`notification_id`, `session_id`, `device_id`) and `content-available: 1` flag via HTTP/2 + ES256 JWT to `api.push.apple.com` or `api.sandbox.push.apple.com`
8. **iOS app** receives APNs push in `didReceiveRemoteNotification`, marks notification_id as "received via push" with 10-minute retention window, then immediately triggers poll for instant UI update
9. **iOS app** polling detects notification_id in push-received list, skips firing duplicate local notification banner (deduplication), but updates bell badge and session highlights
10. **claudiator-server** on APNs 410 Gone response, automatically removes the stale token from the `push_tokens` table
11. **Mobile apps** also poll `/api/v1/ping` every 10s as fallback, detect `notification_version` change, fetch new notifications via `GET /api/v1/notifications`

## Payload Shape

```
Hook stdin (from Claude Code)        Outbound payload (to Server)
─────────────────────────────        ────────────────────────────
{                                    {
  "session_id": "abc123",              "device": {
  "hook_event_name": "Notification",     "device_id": "...",
  "cwd": "/Users/.../project",          "device_name": "shahads-macbook",
  "notification_type": "...",            "platform": "mac"
  "message": "..."                     },
}                                      "event": { ...stdin fields... },
                                       "timestamp": "2026-02-02T15:30:00Z"
                                     }
```

## Server Architecture

**claudiator-server** is a Rust HTTP server built with:
- **Axum** web framework for routing and middleware
- **SQLite** with rusqlite (bundled libsqlite3) for data storage
- **r2d2** connection pooling for concurrent request handling
- **WAL mode** enabled for better concurrent read performance
- **Foreign keys** enabled for referential integrity

### Database Schema

- **devices** — device_id (PK), device_name, platform, first_seen, last_seen
- **sessions** — session_id (PK), device_id (FK), started_at, last_event, status, cwd, title
- **events** — id (PK), device_id (FK), session_id (FK), hook_event_name, timestamp, received_at, tool_name, notification_type, event_json
- **push_tokens** — id (PK), platform, push_token (UNIQUE), sandbox, created_at, updated_at
- **notifications** — id (TEXT PK, UUID), event_id (FK), session_id (FK), device_id (FK), title, body, notification_type, payload_json, acknowledged (BOOLEAN), created_at (24h TTL auto-cleanup)
- **notification_metadata** — notification_id (FK), key (TEXT), value (TEXT), unique constraint on (notification_id, key)

### Server Configuration

Environment variables (stored in `/opt/claudiator/.env`):
- `CLAUDIATOR_API_KEY` — Bearer token for authentication
- `CLAUDIATOR_PORT` — HTTP listen port (default: 3000)
- `CLAUDIATOR_BIND` — Bind address (default: 0.0.0.0)
- `CLAUDIATOR_DB_PATH` — Path to SQLite database (default: /opt/claudiator/claudiator.db)
- `CLAUDIATOR_APNS_KEY_PATH` — Path to .p8 key file (optional)
- `CLAUDIATOR_APNS_KEY_ID` — APNs Key ID (optional)
- `CLAUDIATOR_APNS_TEAM_ID` — Apple Team ID (optional)
- `CLAUDIATOR_APNS_BUNDLE_ID` — App bundle ID (optional)
- `CLAUDIATOR_APNS_SANDBOX` — Use sandbox APNs endpoint (default: false)

### Server Endpoints

- `GET /api/v1/ping` — Health check, returns `data_version` and `notification_version` (requires Bearer auth)
- `POST /api/v1/events` — Ingest hook events, generates notifications for Stop/Notification events (requires Bearer auth)
- `GET /api/v1/devices` — List all devices with active session counts
- `GET /api/v1/devices/:device_id/sessions` — List sessions for a device
- `GET /api/v1/sessions/:session_id/events` — List events for a session
- `GET /api/v1/notifications?after=<uuid>&limit=N` — List notifications after a given UUID
- `POST /api/v1/notifications/:id/ack` — Mark notification as acknowledged
- `POST /api/v1/push/register` — Register mobile push notification token with sandbox flag for APNs routing

### Deployment

The server is deployed as a systemd service on Linux:
- Binary installed to `/opt/claudiator/claudiator-server`
- Database at `/opt/claudiator/claudiator.db`
- Service runs as `claudiator` user
- Logs via journald (`journalctl -u claudiator.service`)

## Key Constraints

### Hook Constraints
- **Hook must never block Claude Code** — 3s HTTP timeout, always exits 0
- **Hook must never write to stderr** — Claude Code captures stderr; errors go to `error.log` only
- **No async runtime** — ureq keeps binary small (~2MB) and startup instant

### Server Constraints
- **Bundled SQLite** — no external database dependencies, single-file storage
- **Systemd deployment** — Linux-first deployment model with service management
- **WAL mode** — enables concurrent reads while maintaining data integrity
- **Connection pooling** — r2d2 manages SQLite connections for multi-threaded Axum

### Notification Constraints
- **Dual-path deduplication** — iOS tracks push-received notification IDs with 10-minute retention to prevent duplicate banners between APNs and polling paths
- **UUID deduplication** — Same notification UUID used across polling and APNs push paths; iOS deduplicates by `UNNotificationRequest.identifier`
- **Immediate poll trigger** — APNs push with `content-available: 1` invokes delegate that immediately triggers poll for instant UI updates (no 10s wait)
- **Enhanced APNs payload** — Push includes custom fields (`notification_id`, `session_id`, `device_id`) for client-side deduplication tracking
- **Polling fallback** — APNs direct push is the primary path; 10s ping polling serves as fallback when push fails or for devices without tokens
- **Non-blocking generation** — Notification records created inside the event transaction; `notification_version` incremented after commit
- **Direct APNs push** — Server sends push notifications directly via HTTP/2 with ES256 JWT authentication
- **Per-token sandbox routing** — Each push token tracks whether it's sandbox or production for correct APNs endpoint routing
- **24h TTL** — Expired notifications are auto-cleaned on each new notification insert

### Future Work
- **Android app** — Native Android (Kotlin) client to consume the server API
- **Web dashboard** — optional browser-based UI for multi-device monitoring
