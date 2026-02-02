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
│   ├── logger.rs     — Error logging
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
```

### Fields

- `server_url` — Base URL of the Claudiator server
- `api_key` — Authentication key for the server
- `device_name` — Human-readable device name
- `device_id` — Unique identifier for this device
- `platform` — Operating system platform (e.g., "darwin", "linux", "windows")

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
    "SessionStart": [{ "matcher": "", "command": "~/.claude/claudiator/claudiator-hook send" }],
    "SessionEnd": [{ "matcher": "", "command": "~/.claude/claudiator/claudiator-hook send" }],
    "Stop": [{ "matcher": "", "command": "~/.claude/claudiator/claudiator-hook send" }],
    "Notification": [{ "matcher": "", "command": "~/.claude/claudiator/claudiator-hook send" }],
    "UserPromptSubmit": [{ "matcher": "", "command": "~/.claude/claudiator/claudiator-hook send" }]
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

## License

MIT
