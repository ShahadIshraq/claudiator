# Claudiator — Project Status & Roadmap

## Overview

Claudiator is a system that pushes Claude Code session events to a central server, enabling mobile notifications when sessions need input or finish. The project consists of a Rust CLI hook, a server backend, installation tooling, and planned mobile applications.

## Project Status

### ✅ Completed Components

#### Hook Binary (`hook/`)
The `claudiator-hook` Rust CLI is complete and production-ready:

- Invoked by Claude Code hooks to capture session events
- Reads hook event JSON from stdin, enriches with device metadata
- POSTs events to the server via HTTP
- Config stored in `~/.claude/claudiator/config.toml` (server_url, api_key, device_name, device_id, platform)
- Subcommands: `send`, `test`, `version`
- Always exits 0 in `send` mode (errors logged to `error.log`)
- 3-second HTTP timeout to avoid blocking Claude Code
- Uses ureq for HTTP (no async runtime, small binary ~2MB)
- Forward-compatible with `#[serde(flatten)]` for unknown fields

#### Hook Install Script (`hook/scripts/install.sh`)
Automated installation script for end users:

- Supports macOS and Linux with architecture detection (x86_64, aarch64)
- Downloads binary from GitHub Releases
- Interactive prompts for server URL and API key
- Generates unique device ID and writes `config.toml`
- Optional auto-configuration of Claude Code hooks in `~/.claude/settings.json`
- JSON manipulation via jq or python3 fallback

#### Hook CI/CD (`.github/workflows/release.yml`)
Automated release pipeline:

- Triggers on `v*` tags
- Cross-compiles for 5 targets: macOS (x86_64, aarch64), Linux (x86_64, aarch64), Windows (x86_64)
- Creates GitHub Release with `.tar.gz` and `.zip` assets

#### Server (`server/`)
Production-ready Rust HTTP server:

- Built with Axum framework
- Endpoints: `GET /api/v1/ping`, `POST /api/v1/events`, `GET /api/v1/devices`, `GET /api/v1/devices/:id/sessions`, `GET /api/v1/sessions`, `GET /api/v1/sessions/:id/events`, `POST /api/v1/push/register`, `GET /api/v1/notifications`, `POST /api/v1/notifications/ack`
- Bearer token authentication on all endpoints
- SQLite database with r2d2 connection pooling (WAL mode enabled)
- Database schema:
  - `devices` table: device metadata and last-seen tracking
  - `sessions` table: session lifecycle management
  - `events` table: all hook events with proper indexes
  - `push_tokens` table: mobile push notification tokens, sandbox
  - `notifications` table: UUID primary key, 24h TTL auto-cleanup
  - Foreign keys and indexes for query performance
- Configuration via CLI args or environment variables:
  - `CLAUDIATOR_API_KEY`: Server authentication token
  - `CLAUDIATOR_PORT`: HTTP port (default: 3000)
  - `CLAUDIATOR_BIND`: Bind address (default: 0.0.0.0)
  - `CLAUDIATOR_DB_PATH`: SQLite database path
  - `CLAUDIATOR_APNS_KEY_PATH`: Path to APNs .p8 key file
  - `CLAUDIATOR_APNS_KEY_ID`: APNs key identifier
  - `CLAUDIATOR_APNS_TEAM_ID`: Apple Developer Team ID
  - `CLAUDIATOR_APNS_BUNDLE_ID`: iOS app bundle identifier
  - `CLAUDIATOR_APNS_SANDBOX`: Set to "true" for sandbox APNs environment
- Uses `rusqlite` with bundled SQLite (no system dependency required)

#### Server Release CI/CD (`.github/workflows/server-release.yml`)
Server deployment automation:

- Triggers on `server-v*` tags
- Linux builds: x86_64 and aarch64
- Cross-compilation support for ARM64
- Creates GitHub Release with `.tar.gz` assets

#### Server Install Script (`server/scripts/install.sh`)
Remote server deployment script:

- Linux-only installer for VPS/cloud deployment
- Installs to `/opt/claudiator/` with systemd service integration
- Creates `claudiator` system user (nologin) for security
- Interactive configuration:
  - API key generation
  - Port and bind address setup
  - Database path configuration
- Writes `.env` file (chmod 600 for security)
- Systemd unit with automatic restart and logging
- Upgrade path: preserves config, replaces binary, restarts service
- Health check validation via `/api/v1/ping` endpoint

#### iOS App (`ios/`)
Production-ready native iOS application:

- **Platform**: SwiftUI, iOS 17+, zero external dependencies
- **Project Setup**: XcodeGen with `project.yml` for reproducible builds
- **Architecture**: MVVM with `@Observable` (modern Swift concurrency)
- **Core Features**:
  - Devices tab: List all devices with last-seen timestamps, active session badges
  - Sessions tab: Cross-device session aggregation view
  - Session detail: Full event timeline with color-coded event types
  - Settings tab: Server URL, API key, theme, appearance
  - Pull-to-refresh on all lists
  - Auto-refresh every 10 seconds
- **Session Titles**: Displays first user prompt (truncated to 200 chars) as session title
- **Theme System**: Four color schemes (Standard, Neon Ops, Solarized, Arctic)
- **Appearance**: Dark mode, light mode, system automatic
- **Security**: API key stored in Keychain, server URL in UserDefaults
- **Platform Icons**: SVG-based device icons (Apple logo, Linux Tux, Windows logo)
- **Notifications**: Dual-path notification system with APNs push and polling fallback
  - **APNs Push**: Direct push from server with custom payload (notification_id, session_id, device_id)
  - **Deduplication**: 10-minute retention window for push-received IDs prevents duplicate banners
  - **Immediate Updates**: Push triggers instant poll for real-time UI refresh (no 10s delay)
  - **Background Handling**: `content-available` flag enables delegate calls when app backgrounded
  - **Polling Fallback**: `notification_version` tracking via ping for reliable delivery
  - **UI Indicators**:
    - Bell icon with badge count in navigation bar
    - Session cards pulse brightness (4-12%) when containing unread notifications
    - Group containers pulse opacity (30-70%) when containing unread notifications
    - Smooth 1.2s breathing animation cycle for attention
  - **Notification List**: Full-screen sheet with swipe-to-mark-read gestures
  - **Theme Integration**: Adaptive notification colors for all themes (light/dark aware)
  - **Auto-dismiss**: Viewing a session automatically marks its notifications as read
- **UI**: Native SwiftUI with iOS design patterns

### 🚧 In Progress / Planned

#### Android App
Native Android application for mobile notifications:

- **Platform**: Native Android (Kotlin)
- **Server Integration**: REST API client for Claudiator server
- **Core Features**:
  - Device list view with last-seen timestamps
  - Live session status per device
  - Session detail view with event history
  - Real-time updates when events occur
- **Push Notifications** via Firebase Cloud Messaging (FCM):
  - `Notification` events (permission prompts, idle prompts, elicitation dialogs)
  - `Stop` events (agent finished, waiting for input)
  - `SessionEnd` events (optional/configurable)
- **UI**: Material Design 3
- **Auth**: Bearer token configuration

#### Server Enhancements (Mobile App Support)
Additional server functionality for mobile apps:

- **Read API Endpoints** (✅ Complete):
  - `GET /api/v1/devices` — list devices with active session counts
  - `GET /api/v1/devices/:device_id/sessions` — list sessions for a device
  - `GET /api/v1/sessions/:session_id/events` — list events for a session
- **Session Titles** (✅ Complete):
  - First `UserPromptSubmit` event stored as `title` in sessions table
  - COALESCE logic prevents overwriting existing titles
  - Truncated to 200 characters for display
- **Push Token Registration** (✅ Complete):
  - `POST /api/v1/push/register` endpoint for mobile push tokens
  - `push_tokens` table in database with `sandbox` column
  - Upsert semantics for token re-registration
- **Notification System** (✅ Complete):
  - [x] `notifications` table with UUID primary key (24h TTL, auto-cleaned)
  - [x] Notification generation on event ingestion (Stop, permission_prompt, idle_prompt)
  - [x] `notification_version` atomic counter in AppState
  - [x] `notification_version` in ping response
  - [x] `GET /api/v1/notifications?after=<timestamp>&limit=N` endpoint (timestamp-based pagination)
  - [x] `POST /api/v1/notifications/ack` endpoint for bulk acknowledging notifications
  - [x] `acknowledged` boolean column in notifications table
  - [x] `metadata` table for persisting version counters across restarts
  - [x] Direct APNs push from server (JWT ES256, HTTP/2, per-token sandbox routing)
  - [x] Device token registration with sandbox flag
  - [x] Install script APNs configuration prompts
- **Live Updates** (optional):
  - WebSocket or Server-Sent Events (SSE) for real-time updates
  - Alternative to polling for session status changes

## Hook Events Captured

| Event | Meaning | Mobile Notification? |
|---|---|---|
| SessionStart | Session begins or resumes | No (tracking only) |
| SessionEnd | Session terminates | Optional |
| Stop | Agent finished, waiting for next prompt | Yes |
| Notification | Permission prompt, idle, dialog | Yes |
| UserPromptSubmit | User submitted a prompt | No (activity tracking) |

## Architecture

```
┌─────────────────┐
│  Claude Code    │
│  (Desktop)      │
└────────┬────────┘
         │ Hook events via stdin
         ↓
┌─────────────────┐
│ claudiator-hook │ (Rust CLI)
│                 │
│ • Enriches with │
│   device metadata
│ • 3s timeout    │
└────────┬────────┘
         │ HTTPS POST
         ↓
┌─────────────────┐
│  Server (Axum)  │
│                 │
│ • SQLite DB     │
│ • Bearer auth   │
│ • Event storage │
└────────┬────────┘
         │ APNs direct push
         ↓
┌─────────────────┐     ┌─────────────────┐
│  Android App    │     │    iOS App      │
│  (Planned)      │     │    (SwiftUI)    │
└─────────────────┘     └─────────────────┘
```

## Installation Paths

### User Machine (Hook)
```
~/.claude/claudiator/
├── claudiator-hook             # Binary
├── config.toml                 # Server URL, API key, device metadata
└── error.log                   # Created on first error
```

### Server Machine
```
/opt/claudiator/
├── claudiator-server           # Binary
├── .env                        # Config (chmod 600)
├── claudiator.db               # SQLite database
└── claudiator.db-wal           # Write-ahead log
```

## Key Design Decisions

- **ureq over reqwest**: No async runtime needed. Binary stays ~2MB vs ~8MB. Startup is instant.
- **Always exit 0 in `send`**: Non-zero exits disrupt Claude Code (exit 2 blocks tool calls).
- **3-second HTTP timeout**: Prevents blocking Claude Code if server is unreachable.
- **`#[serde(flatten)]` for unknown fields**: Forward-compatible with future Claude Code hook changes.
- **Errors to log file only**: Never to stderr during `send` (Claude Code captures stderr).
- **SQLite with WAL mode**: Single-file database with good concurrent read performance.
- **Bearer token auth**: Simple and secure API authentication.
- **Systemd integration**: Automatic startup, restart on failure, log management.

## Roadmap

### Phase 1: Mobile App Foundation
- [ ] Android app development
  - [ ] Project setup and dependencies
  - [ ] API client implementation
  - [ ] Device list UI
  - [ ] Session detail UI
- [x] iOS app development
  - [x] Project setup with XcodeGen
  - [x] API client implementation
  - [x] Device list UI with Devices tab
  - [x] Session detail UI with event timeline
  - [x] Cross-device Sessions tab
  - [x] Settings UI with theme and appearance
  - [x] Session titles from first user prompt
  - [x] Platform-specific device icons
  - [x] Pull-to-refresh and auto-refresh

### Phase 2: Push Notifications
- [x] Server notification infrastructure
  - [x] `notifications` table with UUID primary key (24h TTL, auto-cleaned)
  - [x] Notification generation on event ingestion (Stop, permission_prompt, idle_prompt)
  - [x] `notification_version` atomic counter in AppState
  - [x] `notification_version` in ping response
  - [x] `GET /api/v1/notifications?after=<timestamp>&limit=N` endpoint (timestamp-based pagination)
  - [x] `POST /api/v1/notifications/ack` endpoint for bulk acknowledging notifications
  - [x] `acknowledged` boolean column in notifications table
  - [x] `metadata` table for persisting version counters across restarts
  - [x] Direct APNs push from server (JWT ES256, HTTP/2, per-token sandbox routing)
  - [x] Device token registration with sandbox flag
  - [x] Install script APNs configuration prompts
- [x] iOS notification infrastructure
  - [x] `NotificationService` with permission request and token registration
  - [x] `AppDelegate` handles `didReceiveRemoteNotificationsWithDeviceToken`
  - [x] Auto-detects sandbox vs production environment
  - [x] Registers token with server including sandbox flag
  - [x] `NotificationManager` service (fetch, dedup, local notification firing, 10-minute push deduplication window)
  - [x] `AppNotification` model with acknowledged field
  - [x] `VersionMonitor` extended to track `notification_version`
  - [x] `APIClient` updated: ping returns tuple, new fetch/ack methods
- [x] iOS notification UI
  - [x] Bell icon with badge count in navigation toolbar
  - [x] Notification list sheet view with swipe-to-acknowledge gesture
  - [x] Session card highlight (brightness pulse for sessions with unread notifications)
  - [x] Group container highlight (opacity pulse for groups with unread notifications)
  - [x] Clear highlight on session navigation (auto-acknowledge)
  - [x] Foreground notification banners via `UNUserNotificationCenterDelegate`

### Phase 3: Live Updates (Optional)
- [ ] Server WebSocket/SSE support
- [ ] Android real-time updates
- [ ] iOS real-time updates

### Phase 4: Distribution & Polish
- [ ] TestFlight beta distribution setup
- [ ] App Store preparation and screenshots
- [ ] User onboarding improvements
- [ ] Documentation updates

### Phase 5: Android Development
- [ ] Testing across platforms
- [ ] Documentation updates
- [ ] App store submissions (iOS/Android)
- [ ] User onboarding improvements
