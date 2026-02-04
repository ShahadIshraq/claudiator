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
|  - SessionEnd    |         |  POSTs to server    |         |                  |
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
                                                                      v
                                                            +--------------------+
                                                            |   Mobile Apps      |
                                                            |                    |
                                                            |  iOS (SwiftUI) ✅  |
                                                            |  Android (Planned) |
                                                            |                    |
                                                            |  Live session      |
                                                            |  status, themes,   |
                                                            |  hybrid notifs     |
                                                            +--------------------+
                                                                      |
                                                            (future)  |  paid tier
                                                                      v
                                                            +--------------------+
                                                            |  APNs Push Proxy   |
                                                            | push.claudiator.com|
                                                            |                    |
                                                            |  JWT signing +     |
                                                            |  APNs HTTP/2       |
                                                            +--------------------+
```

## Data Flow

1. **Claude Code** fires a hook event (e.g. Notification) and pipes JSON to stdin
2. **claudiator-hook** reads stdin, parses the event, loads device config from `config.toml`
3. **claudiator-hook** wraps the event in a payload with device info + timestamp
4. **claudiator-hook** POSTs to the server at `POST /api/v1/events` with `Authorization: Bearer {api_key}`
5. **claudiator-server** validates the API key, stores the event in SQLite (devices, sessions, events tables)
6. **claudiator-server** generates a notification record (UUID) for Stop/Notification events, increments `notification_version`
7. **Mobile apps** poll `/api/v1/ping` every 10s, detect `notification_version` change, fetch new notifications
8. **Mobile apps** fire local `UNNotificationRequest` per notification (UUID as identifier for dedup)
9. **Future**: Self-hosted servers POST notifications to APNs push proxy (paid), which dispatches via APNs using same UUID as `apns-collapse-id`

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
- **push_tokens** — id (PK), platform, push_token (UNIQUE), created_at, updated_at
- **notifications** — id (TEXT PK, UUID), event_id (FK), session_id (FK), device_id (FK), title, body, notification_type, payload_json, acknowledged, created_at

### Server Configuration

Environment variables (stored in `/opt/claudiator/.env`):
- `CLAUDIATOR_API_KEY` — Bearer token for authentication
- `CLAUDIATOR_PORT` — HTTP listen port (default: 3000)
- `CLAUDIATOR_BIND` — Bind address (default: 0.0.0.0)
- `CLAUDIATOR_DB_PATH` — Path to SQLite database (default: /opt/claudiator/claudiator.db)

### Server Endpoints

- `GET /api/v1/ping` — Health check, returns `data_version` and `notification_version` (requires Bearer auth)
- `POST /api/v1/events` — Ingest hook events, generates notifications for Stop/Notification events (requires Bearer auth)
- `GET /api/v1/devices` — List all devices with active session counts
- `GET /api/v1/devices/:device_id/sessions` — List sessions for a device
- `GET /api/v1/sessions/:session_id/events` — List events for a session
- `GET /api/v1/notifications?since=<uuid>&limit=N` — List notifications after a given UUID
- `POST /api/v1/notifications/ack` — Acknowledge notifications by ID
- `POST /api/v1/push/register` — Register mobile push notification token (for future APNs proxy)

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
- **UUID deduplication** — Same notification UUID used across polling and future APNs paths; iOS deduplicates by `UNNotificationRequest.identifier`
- **Polling-first** — Free tier relies on 10s ping polling; no APNs keys needed on self-hosted servers
- **Non-blocking generation** — Notification records created inside the event transaction; `notification_version` incremented after commit
- **Future APNs proxy** — Paid service at `push.claudiator.com`; self-hosted servers POST notification payloads, proxy dispatches via APNs with same UUID as `apns-collapse-id`

### Future Work
- **Android app** — Native Android (Kotlin) client to consume the server API
- **APNs push proxy** — Paid service for real-time push notification delivery
- **Web dashboard** — optional browser-based UI for multi-device monitoring
