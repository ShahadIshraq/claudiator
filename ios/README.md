# claudiator-ios

A SwiftUI iOS app for monitoring Claude Code sessions across devices in real-time.

## Overview

Claudiator is a pure SwiftUI mobile client that connects to a Claudiator server to display live session monitoring across all your development machines. It provides a device-centric view of active sessions, event timelines, and customizable themes with automatic refresh and push notification support.

The app requires iOS 17.0+ and has zero external dependencies, using only iOS SDK frameworks.

## Directory Layout

```
ios/
├── project.yml                         — XcodeGen project definition
├── Claudiator.xcodeproj/              — Generated Xcode project
├── Claudiator/
│   ├── ClaudiatorApp.swift            — App entry point (@main)
│   ├── Info.plist                     — UIKit configuration
│   ├── Claudiator.entitlements        — Push notification capability
│   ├── Assets.xcassets/               — App icon and image assets
│   │   ├── AppIcon.appiconset/        — App icon (1024x1024)
│   │   ├── ClaudiatorLogo.imageset/   — In-app logo (1x/2x/3x)
│   │   ├── AppleLogo.imageset/        — macOS platform icon (SVG)
│   │   ├── LinuxLogo.imageset/        — Linux platform icon (SVG)
│   │   └── WindowsLogo.imageset/      — Windows platform icon (SVG)
│   ├── Models/
│   │   ├── Device.swift               — Device data model
│   │   ├── Session.swift              — Session data model (includes title)
│   │   ├── Event.swift                — Event data model
│   │   └── Extensions.swift           — Codable/Hashable extensions
│   ├── Services/
│   │   ├── APIClient.swift            — REST API client (URLSession, async/await)
│   │   ├── KeychainService.swift      — Secure credential storage
│   │   └── NotificationService.swift  — APNs push notification registration
│   ├── Theme/
│   │   ├── AppTheme.swift             — Theme protocol and color definitions
│   │   ├── AppTheme+Themes.swift      — Theme variants (Standard, Neon Ops, Solarized, Arctic)
│   │   └── ThemeManager.swift         — Observable theme state manager
│   ├── ViewModels/
│   │   ├── AllSessionsViewModel.swift — Cross-device session aggregation
│   │   ├── DeviceListViewModel.swift  — Device list state
│   │   ├── EventListViewModel.swift   — Event timeline state
│   │   ├── SessionListViewModel.swift — Per-device session state
│   │   └── SetupViewModel.swift       — Onboarding/setup flow state
│   └── Views/
│       ├── SetupView.swift            — Server URL + API key onboarding
│       ├── DeviceListView.swift       — Device list tab
│       ├── DeviceDetailView.swift     — Device sessions view
│       ├── AllSessionsView.swift      — Cross-device sessions tab
│       ├── SessionDetailView.swift    — Session detail with event timeline
│       ├── SessionRow.swift           — Session list row component
│       ├── EventRow.swift             — Event timeline row component
│       ├── SettingsView.swift         — Settings tab (theme, server config)
│       └── Helpers.swift              — Shared display utilities
```

## Prerequisites

- Xcode 16+
- iOS 17.0+ (target device or simulator)
- XcodeGen (for project generation)

### Installing XcodeGen

```bash
brew install xcodegen
```

## Build & Run

### 1. Generate Xcode Project

```bash
cd ios
xcodegen generate
```

This creates `Claudiator.xcodeproj` from the `project.yml` spec.

### 2. Open in Xcode

```bash
open Claudiator.xcodeproj
```

### 3. Build and Run

Select a simulator or device target, then press `Cmd+R` to build and run.

## Architecture

The app follows MVVM architecture with modern SwiftUI patterns:

- **Models** — Codable structs matching the Claudiator server API schema
- **ViewModels** — Observable state containers using `@Observable` macro (Swift 5.9+)
- **Views** — Pure SwiftUI declarative UI
- **Services** — Async/await networking with URLSession, keychain storage, APNs integration

### Environment Injection

Core services are injected into the SwiftUI environment at the root:

```swift
ClaudiatorApp
  .environmentObject(ThemeManager.shared)
  .environmentObject(APIClient.shared)
```

ViewModels are instantiated per-view and consume these services.

## Features

### Session Display

Session titles are displayed with a fallback priority:

1. **Title** — First user prompt submitted in the session
2. **CWD** — Working directory if no title available
3. **Session ID** — As a last resort

This provides meaningful context for each session at a glance.

### Themes

Four built-in themes with full dark/light mode support:

- **Standard** — Default iOS-style colors
- **Neon Ops** — High-contrast cyberpunk palette
- **Solarized** — Classic Solarized color scheme
- **Arctic** — Cool blue/white tones

Theme selection persists in UserDefaults and applies system-wide appearance preferences (light/dark/auto).

### Auto-Refresh

All views refresh every 10 seconds to display live session updates without manual intervention.

### Push Notifications

APNs push token registration is handled via the `NotificationService`. Tokens are registered with the Claudiator server on successful authorization.

Requires the `aps-environment` entitlement (included in `Claudiator.entitlements`).

### Tab Navigation

- **Devices** — List of all devices with active session counts and platform icons
- **Sessions** — Aggregated view of all sessions across devices
- **Settings** — Theme selection, appearance mode, server configuration

## Configuration

### First Launch Setup

On first launch, the app presents a setup view (`SetupView`) to collect:

- **Server URL** — Base URL of the Claudiator server (e.g., `https://api.claudiator.example.com`)
- **API Key** — Bearer token for API authentication

These credentials are validated with a ping request and stored securely in the keychain via `KeychainService`.

### Reconfiguration

To change server settings, navigate to **Settings > Server Configuration** and tap "Reconfigure". This clears stored credentials and returns to the setup flow.

## API Communication

All API requests use Bearer token authentication:

```
Authorization: Bearer <api_key>
```

### Endpoints Used

- `GET /api/v1/ping` — Health check and version info
- `GET /api/v1/devices` — List all devices
- `GET /api/v1/devices/:device_id/sessions` — Sessions for a device
- `GET /api/v1/sessions/:session_id/events` — Events for a session
- `POST /api/v1/push/register` — Register APNs token

See the [server API documentation](../server/API.md) for full request/response schemas.

## Development

### Running Locally

1. Ensure a Claudiator server is running (see [server README](../server/README.md))
2. Generate the Xcode project: `xcodegen generate`
3. Open in Xcode: `open Claudiator.xcodeproj`
4. Select a simulator and run

### Project Regeneration

Any changes to `project.yml` require regenerating the Xcode project:

```bash
xcodegen generate
```

The generated `.xcodeproj` is excluded from version control. Only `project.yml` is committed.

## License

MIT
