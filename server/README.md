# claudiator-server

A Rust HTTP server that ingests Claude Code hook events, stores them in SQLite, and serves a REST API for monitoring sessions across devices.

## Overview

`claudiator-server` is the backend component of Claudiator. It receives events from `claudiator-hook` clients running on developer machines, persists them in a SQLite database, and exposes read endpoints for mobile apps or other consumers. It also supports push notification token registration and APNs push notification delivery for mobile devices.

## Directory Layout

```
server/
├── Cargo.toml
├── API.md                  — Full API contract documentation
├── README.md
├── src/
│   ├── main.rs             — Entry point, server initialization
│   ├── config.rs           — CLI/env configuration (clap)
│   ├── router.rs           — Route definitions and AppState
│   ├── auth.rs             — Bearer token authentication
│   ├── error.rs            — Error types and responses
│   ├── apns.rs             — APNs client (JWT auth, HTTP/2 push delivery)
│   ├── db/
│   │   ├── mod.rs
│   │   ├── pool.rs         — r2d2 connection pool setup
│   │   ├── migrations.rs   — Schema creation (devices, sessions, events, push_tokens, notifications)
│   │   └── queries.rs      — SQL query functions
│   ├── models/
│   │   ├── mod.rs
│   │   ├── request.rs      — Request payload structs
│   │   └── response.rs     — Response payload structs
│   └── handlers/
│       ├── mod.rs
│       ├── ping.rs          — GET /api/v1/ping
│       ├── events.rs        — POST /api/v1/events
│       ├── devices.rs       — GET /api/v1/devices, GET /api/v1/devices/:id/sessions
│       ├── sessions.rs      — GET /api/v1/sessions/:id/events
│       ├── push.rs          — POST /api/v1/push/register
│       └── notifications.rs — GET /api/v1/notifications
└── scripts/
    ├── install.sh           — Linux/systemd installer
    ├── update.sh            — Non-interactive updater
    └── seed.sh              — Test data seeder
```

## Build

```bash
cargo build --release
```

The binary will be available at `target/release/claudiator-server`.

### Dependencies

- **axum** — HTTP framework
- **tokio** — Async runtime
- **rusqlite** (bundled) — SQLite driver, no system dependency required
- **r2d2** — Connection pooling
- **clap** — CLI argument parsing
- **chrono** — Timestamp handling
- **serde/serde_json** — Serialization
- **tracing** — Structured logging
- **uuid** — Notification ID generation
- **jsonwebtoken** — APNs JWT ES256 signing
- **reqwest** — APNs HTTP/2 client

## Running

```bash
claudiator-server --api-key <key> [--port 3000] [--bind 0.0.0.0] [--db-path claudiator.db]
```

Or via environment variables:

```bash
export CLAUDIATOR_API_KEY=your-secret-key
export CLAUDIATOR_PORT=3000
export CLAUDIATOR_BIND=0.0.0.0
export CLAUDIATOR_DB_PATH=/opt/claudiator/data/claudiator.db
claudiator-server
```

### Configuration

| Flag / Env Var | Default | Description |
|---|---|---|
| `--api-key` / `CLAUDIATOR_API_KEY` | (required) | Bearer token for API authentication |
| `--port` / `CLAUDIATOR_PORT` | `3000` | HTTP listen port |
| `--bind` / `CLAUDIATOR_BIND` | `0.0.0.0` | Bind address |
| `--db-path` / `CLAUDIATOR_DB_PATH` | `claudiator.db` | Path to SQLite database file |
| `--apns-key-path` / `CLAUDIATOR_APNS_KEY_PATH` | — | Path to APNs .p8 authentication key |
| `--apns-key-id` / `CLAUDIATOR_APNS_KEY_ID` | — | APNs Key ID (10-character string) |
| `--apns-team-id` / `CLAUDIATOR_APNS_TEAM_ID` | — | Apple Developer Team ID |
| `--apns-bundle-id` / `CLAUDIATOR_APNS_BUNDLE_ID` | — | iOS app bundle identifier |
| `--apns-sandbox` / `CLAUDIATOR_APNS_SANDBOX` | `false` | Use APNs sandbox endpoint |
| `--log-level` / `CLAUDIATOR_LOG_LEVEL` | `info` | Log level (debug, info, warn, error) |
| `--log-dir` / `CLAUDIATOR_LOG_DIR` | `logs` | Log directory (daily rotation) |
| `--retention-events-days` / `CLAUDIATOR_RETENTION_EVENTS_DAYS` | `7` | Days to retain events |
| `--retention-sessions-days` / `CLAUDIATOR_RETENTION_SESSIONS_DAYS` | `7` | Days to retain sessions |
| `--retention-devices-days` / `CLAUDIATOR_RETENTION_DEVICES_DAYS` | `30` | Days to retain devices |

The database file and WAL files are created automatically on first run.

**Note:** APNs configuration is optional. Without it, the server operates normally but does not send push notifications. See [APNS_SETUP.md](APNS_SETUP.md) for a step-by-step setup guide.

## API Endpoints

All endpoints require `Authorization: Bearer <api_key>`.

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/ping` | Health check, returns server version, data_version, and notification_version |
| `POST` | `/api/v1/events` | Ingest a hook event from a device |
| `GET` | `/api/v1/devices` | List all devices with active session counts |
| `GET` | `/api/v1/devices/:device_id/sessions` | List sessions for a device |
| `GET` | `/api/v1/sessions` | List all sessions across all devices |
| `GET` | `/api/v1/sessions/:session_id/events` | List events for a session |
| `POST` | `/api/v1/push/register` | Register a mobile push notification token |
| `GET` | `/api/v1/notifications` | List notifications (with optional `after` and `limit` params) |
| `POST` | `/api/v1/notifications/ack` | Bulk acknowledge notifications (accepts `ids` array) |

See [API.md](API.md) for full request/response schemas and query parameters.

## Database

SQLite with WAL mode enabled. The schema is created automatically on startup.

### Tables

- **devices** — Device metadata and last-seen tracking
- **sessions** — Session lifecycle (status, cwd, title, timestamps)
- **events** — All hook events with full JSON storage
- **push_tokens** — Mobile push notification tokens (APNs/FCM) with sandbox tracking
- **notifications** — Push notification records (UUID primary key, 24h TTL auto-cleanup, acknowledged boolean column)
- **metadata** — Key-value store for persistent counters (data_version, notification_version)

### Session Status Values

Status is derived from hook events:

| Hook Event | Derived Status |
|---|---|
| `SessionStart`, `UserPromptSubmit` | `active` |
| `SubagentStart`, `SubagentStop` | `active` |
| `Stop` | `waiting_for_input` |
| `SessionEnd` | `ended` |
| `PermissionRequest` | `waiting_for_permission` |
| `Notification` (permission_prompt) | `waiting_for_permission` |
| `Notification` (idle_prompt) | `idle` |

### Session Title

The first `UserPromptSubmit` event in a session sets the session title from the user's prompt text (truncated to 200 characters). Subsequent prompts do not overwrite the title.

## Deployment

### Quick Start (Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/shahadishraq/claudiator/main/server/scripts/install.sh | sudo bash
```

The install script:

1. Downloads the latest release binary for your architecture (x86_64 or aarch64)
2. Creates `/opt/claudiator/` directory and `claudiator` system user
3. Prompts for configuration (API key, port, bind address, database path)
4. Writes `.env` file (chmod 600)
5. Installs and enables a systemd service
6. Runs a health check

### File Layout (Production)

```
/opt/claudiator/
├── claudiator-server           — Binary
├── update.sh                   — Update script
├── .env                        — Config (chmod 600)
└── data/
    ├── claudiator.db           — SQLite database
    ├── claudiator.db-wal       — Write-ahead log
    └── claudiator.db-shm       — Shared memory
```

### Service Management

```bash
systemctl status claudiator-server
systemctl restart claudiator-server
journalctl -u claudiator-server -f
```

### Upgrading

#### Update Script (Recommended)

```bash
sudo /opt/claudiator/update.sh
```

The update script checks GitHub for the latest server release, downloads and replaces the binary, restarts the service, and runs a health check. If the health check fails, it automatically rolls back to the previous version.

To check for updates without applying them:

```bash
sudo /opt/claudiator/update.sh --check
```

#### Re-run Install Script

Alternatively, re-run the install script. It detects existing installations, preserves configuration, replaces the binary, and restarts the service.

### Uninstalling

```bash
sudo systemctl stop claudiator-server
sudo systemctl disable claudiator-server
sudo rm /etc/systemd/system/claudiator-server.service
sudo systemctl daemon-reload
sudo userdel claudiator
sudo rm -rf /opt/claudiator
```

## Development

### Running Locally

```bash
CLAUDIATOR_API_KEY=test-key cargo run
```

**Note:** To test push notifications locally, also set the `CLAUDIATOR_APNS_*` environment variables.

### Testing with curl

```bash
# Health check
curl -s -H "Authorization: Bearer test-key" http://localhost:3000/api/v1/ping

# Send a test event
curl -s -X POST http://localhost:3000/api/v1/events \
  -H "Authorization: Bearer test-key" \
  -H "Content-Type: application/json" \
  -d '{
    "device": {"device_id": "dev-001", "device_name": "my-laptop", "platform": "mac"},
    "event": {"session_id": "sess-001", "hook_event_name": "SessionStart", "cwd": "/Users/me/project"},
    "timestamp": "2026-01-15T10:00:00.000Z"
  }'

# List devices
curl -s -H "Authorization: Bearer test-key" http://localhost:3000/api/v1/devices

# List sessions for a device
curl -s -H "Authorization: Bearer test-key" http://localhost:3000/api/v1/devices/dev-001/sessions

# List events for a session
curl -s -H "Authorization: Bearer test-key" http://localhost:3000/api/v1/sessions/sess-001/events

# List notifications
curl -s -H "Authorization: Bearer test-key" http://localhost:3000/api/v1/notifications

# List notifications after a specific ID
curl -s -H "Authorization: Bearer test-key" "http://localhost:3000/api/v1/notifications?after=<uuid>&limit=10"

# Acknowledge notifications
curl -s -X POST -H "Authorization: Bearer test-key" \
  -H "Content-Type: application/json" \
  -d '{"ids": ["<uuid>"]}' \
  http://localhost:3000/api/v1/notifications/ack
```

### Seed Data

```bash
# Seed test data (3 devices, 5 sessions, 15 events)
./scripts/seed.sh
```

## License

MIT
