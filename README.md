# Claudiator

Monitor your Claude Code sessions from anywhere.

Claudiator is a lightweight system that captures Claude Code hook events and forwards them to a server, letting you track session activity and receive notifications on your mobile device.

## Components

- **claudiator-hook** — A Rust CLI binary that runs as a Claude Code hook. It reads hook events from stdin and POSTs them to the Claudiator server.
- **Claudiator Server** — Stores events, manages devices, and serves the API.
- **Claudiator Mobile App** — Displays live session status and notifications per device.

## How It Works

1. Claude Code fires hook events (SessionStart, SessionEnd, Stop, Notification, PromptSubmit)
2. `claudiator-hook` reads the JSON event from stdin, wraps it with device info, and POSTs it to the server
3. The server stores the event and makes it available to the mobile app
4. You see live session activity on your phone

## Architecture

See [plans/ARCHITECTURE.md](plans/ARCHITECTURE.md) for the full architecture diagram and data flow.

## License

MIT
