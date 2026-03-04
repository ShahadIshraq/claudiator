<p align="center">
  <img src="claudiator-icon-no-bg.png" alt="Claudiator" width="200">
</p>

# Claudiator

[![CI](https://github.com/ShahadIshraq/claudiator/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ShahadIshraq/claudiator/actions/workflows/ci.yml)
[![iOS Quality](https://github.com/ShahadIshraq/claudiator/actions/workflows/ios-quality.yml/badge.svg?branch=main)](https://github.com/ShahadIshraq/claudiator/actions/workflows/ios-quality.yml)

Orchestrate Claude Code sessions across all your devices.

Claudiator is a lightweight system that captures Claude Code hook events and forwards them to a server, giving you a unified view of all your parallel agent sessions across devices. Track what's running where, know which sessions need input, and orchestrate your Claude Code work across multiple machines.

## Components

- **claudiator-hook** — A Rust CLI binary that runs as a Claude Code hook. It reads hook events from stdin and POSTs them to the Claudiator server.
- **Claudiator Server** — Stores events, manages devices, serves the REST API, and sends push notifications directly to APNs.
- **Claudiator iOS App** — A SwiftUI app for monitoring sessions, viewing event timelines, and receiving real-time push notifications via APNs. See [ios/README.md](ios/README.md).

## How It Works

1. Claude Code fires hook events (SessionStart, SessionEnd, Stop, Notification, PromptSubmit)
2. `claudiator-hook` reads the JSON event from stdin, wraps it with device info, and POSTs it to the server
3. The server stores the event and makes it available to the mobile app
4. The server sends push notifications to your iOS device when sessions need attention (stopped, waiting for permission, idle)
5. You see all active sessions across devices in the iOS app and orchestrate parallel agent work

## HTTP Hooks (Direct)

Claude Code can POST hook events directly over HTTP (no local hook binary). Claudiator supports this via `POST /api/v1/hooks/http`, which accepts the raw hook JSON and uses headers for device identity. See [server/API.md](server/API.md) and [hook/README.md](hook/README.md) for the exact payload and headers.

## Migration: Stdin Hook → HTTP Hook

1. Re-run the hook installer and choose `http` when prompted for hook transport:
   - macOS/Linux: `hook/scripts/install.sh`
   - Windows: `hook/scripts/install.ps1`
2. The installer will add HTTP hook entries to `~/.claude/settings.json` using your device metadata and API key.
3. Optionally remove the old `command` hooks if you want only HTTP hooks.

Note: HTTP hooks embed the API key directly in `settings.json`. If you prefer not to store it there, use the stdin hook client instead.

## Using the iOS App

The iOS app's push notifications are powered by APNs credentials tied to my own Apple Developer account. If you want push notification support, you have two options:

1. **Build it yourself** — Clone the repo, set up your own APNs credentials (see [server/APNS_SETUP.md](server/APNS_SETUP.md)), and build via Xcode.
2. **Reach out to me** — I can help set up a dedicated server instance that you control, using my APNs config. Your data stays yours.

## Data Sent to the Server

`claudiator-hook` trims every event to exactly 7 fields before transmission. Everything else — including `tool_input`, `tool_output`, `tool_response`, `custom_instructions`, and `transcript_path` — is discarded on the client machine and never leaves it.

| Field | Purpose |
|---|---|
| `session_id` | Identifies the session |
| `hook_event_name` | Event type (e.g. `Stop`, `Notification`) |
| `cwd` | Working directory shown in the app |
| `prompt` | Session title (from `UserPromptSubmit`) |
| `notification_type` | Notification routing |
| `tool_name` | Shown in notification body |
| `message` | Notification message text |

This is what gets stored in the server database. No file contents, no conversation data, no instructions.

## Architecture

See [plans/ARCHITECTURE.md](plans/ARCHITECTURE.md) for the full architecture diagram and data flow.

## Repo Structure

```
claudiator/
├── .github/workflows/               — CI/CD release workflows
├── hook/                             — claudiator-hook CLI binary (Rust)
│   ├── src/                          — Source modules
│   ├── scripts/                      — Install scripts (macOS/Linux/Windows)
│   └── test-server/                  — Local test server (Axum)
├── server/                           — Claudiator server (Rust, Axum + SQLite)
│   ├── src/                          — Server source code
│   └── scripts/                      — Server install script (Linux/systemd)
├── ios/                              — Claudiator iOS app (SwiftUI)
│   ├── Claudiator/                   — App source code
│   └── project.yml                   — XcodeGen project definition
├── plans/                            — Architecture & planning docs
└── README.md
```

See [hook/README.md](hook/README.md) for build instructions, CLI usage, and configuration details.

See [server/README.md](server/README.md) for the server API, deployment, and configuration.

See [ios/README.md](ios/README.md) for the iOS app build instructions and architecture.

## Development

### Pre-commit Hooks

This repository includes a pre-commit hook to enforce code quality. Install it once after cloning:

```bash
./install-hooks.sh
```

This symlinks `.githooks/pre-commit` into `.git/hooks/` — the default location git checks on every commit with no extra config required.

The hook automatically:

**Rust (hook/ and server/):**
- Runs `cargo fmt` and re-stages any reformatted files
- Runs `cargo clippy` with the same flags as CI — blocks the commit on warnings

**iOS (Swift):**
- Runs SwiftFormat (auto-fix) on staged `.swift` files and re-stages them
- Runs SwiftFormat lint on the whole `ios/Claudiator/` tree — blocks on any violation in any file
- Runs SwiftLint (strict) on staged `.swift` files

### Dependency Auditing

Both the hook and server use `cargo-deny` for dependency auditing. Configurations are in `hook/deny.toml` and `server/deny.toml`.

To audit dependencies:
```bash
cargo install cargo-deny
cargo deny --manifest-path hook/Cargo.toml check
cargo deny --manifest-path server/Cargo.toml check
```

## License

MIT
