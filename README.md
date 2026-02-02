<p align="center">
  <img src="claudiator-icon-no-bg.png" alt="Claudiator" width="200">
</p>

# Claudiator

Orchestrate Claude Code sessions across all your devices.

Claudiator is a lightweight system that captures Claude Code hook events and forwards them to a server, giving you a unified view of all your parallel agent sessions across devices. Track what's running where, know which sessions need input, and orchestrate your Claude Code work across multiple machines.

## Components

- **claudiator-hook** — A Rust CLI binary that runs as a Claude Code hook. It reads hook events from stdin and POSTs them to the Claudiator server.
- **Claudiator Server** — Stores events, manages devices, and serves the API.
- **Claudiator Mobile App** — Displays live session status and notifications per device.

## How It Works

1. Claude Code fires hook events (SessionStart, SessionEnd, Stop, Notification, PromptSubmit)
2. `claudiator-hook` reads the JSON event from stdin, wraps it with device info, and POSTs it to the server
3. The server stores the event and makes it available to the mobile app
4. You see all active sessions across devices, know which need attention (waiting for prompts, notifications), and orchestrate parallel agent work

## Architecture

See [plans/ARCHITECTURE.md](plans/ARCHITECTURE.md) for the full architecture diagram and data flow.

## Repo Structure

```
claudiator/
├── .github/workflows/release.yml   — CI/CD release workflow
├── hook/                            — claudiator-hook CLI binary (Rust)
│   ├── src/                         — Source modules
│   ├── scripts/                     — Install scripts (macOS/Linux/Windows)
│   └── test-server/                 — Local test server (Axum)
├── plans/                           — Architecture docs
└── README.md
```

See [hook/README.md](hook/README.md) for build instructions, CLI usage, and configuration details.

## License

MIT
