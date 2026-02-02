# Claudiator — Architecture

## Components

```
+------------------+         +---------------------+         +------------------+
|   Claude Code    |         |  claudiator-hook    |         |   Claudiator     |
|   (IDE/CLI)      |         |  (Rust CLI binary)  |         |   Server         |
|                  |         |                     |         |                  |
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
                             ~/.claude/claudiator/          +--------------------+
                             ├── config.toml                |   Claudiator       |
                             ├── claudiator-hook            |   Web / Mobile App |
                             └── error.log                  |                    |
                                                            |  Displays live     |
                                                            |  session status    |
                                                            |  & notifications   |
                                                            +--------------------+
```

## Data Flow

1. **Claude Code** fires a hook event (e.g. Notification) and pipes JSON to stdin
2. **claudiator-hook** reads stdin, parses the event, loads device config from `config.toml`
3. **claudiator-hook** wraps the event in a payload with device info + timestamp
4. **claudiator-hook** POSTs to the server at `POST /api/v1/events` with `Authorization: Bearer {api_key}`
5. **Server** validates the API key, stores the event, and makes it available to the web app
6. **Web App** displays live session activity and notifications per device

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

## Key Constraints

- **Hook must never block Claude Code** — 3s HTTP timeout, always exits 0
- **Hook must never write to stderr** — Claude Code captures stderr; errors go to `error.log` only
- **No async runtime** — ureq keeps binary small (~2MB) and startup instant
- **Server and App are separate concerns** — this repo only contains the hook binary and install tooling
