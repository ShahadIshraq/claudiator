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
- Runs SwiftFormat (auto-fix) and SwiftLint (strict) on staged `.swift` files

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
