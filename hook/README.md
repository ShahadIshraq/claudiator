# claudiator-hook

A Rust CLI that runs as a Claude Code hook, reading JSON hook events from stdin and forwarding them to a Claudiator server for centralized session monitoring across devices.

## Overview

`claudiator-hook` is the client-side component that captures Claude Code session events and reports them to a Claudiator server, enabling you to monitor all your parallel Claude Code sessions across devices from one central dashboard. It reads hook events from stdin, enriches them with device metadata, and POSTs them via HTTP.

Configuration is loaded from `~/.claude/claudiator/config.toml`.

## Directory Layout

```
hook/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs       — Entry point, dispatches subcommands
│   ├── cli.rs        — CLI argument parser (clap)
│   ├── config.rs     — Config loading from TOML
│   ├── error.rs      — Error types
│   ├── event.rs      — Hook event parsing from stdin
│   ├── logger.rs     — Logging with levels and rotation
│   ├── payload.rs    — Event payload construction
│   └── sender.rs     — HTTP client (ureq)
├── scripts/
│   ├── install.sh    — macOS/Linux installer
│   └── install.ps1   — Windows installer
└── test-server/
    ├── Cargo.toml
    └── src/main.rs   — Axum-based test server
```

## Build

```bash
cargo build --release
```

The binary will be available at `target/release/claudiator-hook`.

## CLI Usage

### Send Event

Read a hook event from stdin and send it to the configured server:

```bash
claudiator-hook send
```

This subcommand is typically called by Claude Code hooks. It reads JSON from stdin, parses the event, and POSTs it to the server.

### Global Options

#### `--log-level <level>`

Override the log level for this invocation. Can be placed before or after the subcommand:

```bash
claudiator-hook --log-level debug send
claudiator-hook send --log-level debug
```

Valid levels: `error`, `warn`, `info`, `debug` (case-insensitive).

### Test Connection

Test connectivity to the configured Claudiator server:

```bash
claudiator-hook test
```

Sends a ping request to verify server availability and authentication.

### Version

Print the version and exit:

```bash
claudiator-hook version
```

## Configuration

Configuration file location: `~/.claude/claudiator/config.toml`

### Format

```toml
server_url = "https://api.claudiator.example.com"
api_key = "your-api-key-here"
device_name = "MacBook Pro"
device_id = "unique-device-identifier"
platform = "mac"

# Logging (optional — defaults shown)
log_level = "error"
max_log_size_bytes = 1048576
max_log_backups = 2
```

### Fields

- `server_url` — Base URL of the Claudiator server
- `api_key` — Authentication key for the server
- `device_name` — Human-readable device name
- `device_id` — Unique identifier for this device
- `platform` — Operating system platform (e.g., "darwin", "linux", "windows")
- `log_level` — Minimum log level: `error`, `warn`, `info`, or `debug` (default: `"error"`)
- `max_log_size_bytes` — Maximum log file size in bytes before rotation (default: `1048576` / 1 MB)
- `max_log_backups` — Number of rotated log files to keep (default: `2`)

## Logging

All log output is written to `~/.claude/claudiator/error.log`. The hook never writes to stderr during `send` mode to avoid interfering with Claude Code.

### Log Levels

| Level | Description |
|-------|-------------|
| `error` | Errors only (default) |
| `warn` | Errors and warnings |
| `info` | Errors, warnings, and informational messages |
| `debug` | All messages including debug details |

### Log Level Precedence

The log level is resolved in this order (first match wins):

1. `--log-level` CLI flag
2. `CLAUDIATOR_LOG_LEVEL` environment variable
3. `log_level` in `config.toml`
4. Default: `error`

### Log Rotation

When the log file exceeds `max_log_size_bytes`, it is rotated:

- `error.log` is renamed to `error.log.1`
- Existing backups shift: `.1` becomes `.2`, etc.
- The oldest backup beyond `max_log_backups` is deleted
- If `max_log_backups` is `0`, the file is truncated instead of rotated

## Test Server

A local test server is provided for development and testing.

### Running

```bash
cd test-server
cargo run -- --port 3000 --api-key test-key
```

### Endpoints

- `GET /api/v1/ping` — Health check endpoint
- `POST /api/v1/events` — Event ingestion endpoint

The test server validates the API key via the `Authorization: Bearer <key>` header and logs all received events to stdout.

## Installation Scripts

Automated installers are provided in the `scripts/` directory:

- `scripts/install.sh` — macOS/Linux installer
- `scripts/install.ps1` — Windows installer

These scripts:

1. Download the latest release binary
2. Prompt for configuration values (server URL, API key, device info)
3. Create the config file at `~/.claude/claudiator/config.toml`
4. Optionally configure Claude Code hooks in `~/.claude/settings.json`

## Claude Code Hook Integration

To integrate with Claude Code, add the hook to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "SessionStart": [{ "matcher": "", "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}] }],
    "SessionEnd": [{ "matcher": "", "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}] }],
    "Stop": [{ "matcher": "", "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}] }],
    "Notification": [{ "matcher": "", "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}] }],
    "UserPromptSubmit": [{ "matcher": "", "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}] }],
    "PermissionRequest": [{ "matcher": "", "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}] }],
    "TeammateIdle": [{ "matcher": "", "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}] }],
    "TaskCompleted": [{ "matcher": "", "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}] }]
  }
}
```

When any of these events occur, Claude Code will invoke `claudiator-hook send` with the event JSON on stdin.

### Supported Events

- `SessionStart` — Fired when a new Claude Code session begins
- `SessionEnd` — Fired when a session ends
- `Stop` — Fired when execution is stopped
- `Notification` — Fired when notifications are generated
- `UserPromptSubmit` — Fired when a user submits a prompt
- `PermissionRequest` — Fired when a tool permission is requested
- `TeammateIdle` — Fired when a teammate agent goes idle
- `TaskCompleted` — Fired when a task is completed

### Opt-in: Sub-Agent Events

Sub-agent hooks (`SubagentStart`, `SubagentStop`) are **not installed by default** because they fire for every spawned sub-agent and can generate significant volume when using teams or parallel agents. To opt in, add them manually to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "SubagentStart": [{ "matcher": "", "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}] }],
    "SubagentStop": [{ "matcher": "", "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}] }]
  }
}
```

## License

MIT
