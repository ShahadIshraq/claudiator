# claudiator-server

A Rust HTTP server that ingests Claude Code hook events, stores them in SQLite, and serves a REST API for monitoring sessions across devices.

## Overview

`claudiator-server` is the backend component of Claudiator. It receives events from `claudiator-hook` clients running on developer machines, persists them in a SQLite database, and exposes read endpoints for mobile apps or other consumers. It also supports push notification token registration for mobile devices.

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
│   ├── db/
│   │   ├── mod.rs
│   │   ├── pool.rs         — r2d2 connection pool setup
│   │   ├── migrations.rs   — Schema creation (devices, sessions, events, push_tokens)
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
│       └── push.rs          — POST /api/v1/push/register
└── scripts/
    └── install.sh           — Linux/systemd installer
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

The database file and WAL files are created automatically on first run.

## API Endpoints

All endpoints require `Authorization: Bearer <api_key>`.

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/ping` | Health check, returns server version |
| `POST` | `/api/v1/events` | Ingest a hook event from a device |
| `GET` | `/api/v1/devices` | List all devices with active session counts |
| `GET` | `/api/v1/devices/:device_id/sessions` | List sessions for a device |
| `GET` | `/api/v1/sessions/:session_id/events` | List events for a session |
| `POST` | `/api/v1/push/register` | Register a mobile push notification token |

See [API.md](API.md) for full request/response schemas and query parameters.

## Database

SQLite with WAL mode enabled. The schema is created automatically on startup.

### Tables

- **devices** — Device metadata and last-seen tracking
- **sessions** — Session lifecycle (status, cwd, title, timestamps)
- **events** — All hook events with full JSON storage
- **push_tokens** — Mobile push notification tokens (APNs/FCM)

### Session Status Values

Status is derived from hook events:

| Hook Event | Derived Status |
|---|---|
| `SessionStart`, `UserPromptSubmit` | `active` |
| `Stop` | `waiting_for_input` |
| `SessionEnd` | `ended` |
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

Re-run the install script. It detects existing installations, preserves configuration, replaces the binary, and restarts the service.

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
```

## License

MIT
