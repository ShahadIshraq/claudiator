# Claudiator iOS App — Technology & Architecture Plan

## Platform Target

- **iOS 17+**, **Swift 5.9+**, **SwiftUI** only
- **Universal app**: iPhone and iPad (single binary, adaptive layouts)
- Directory: `ios/` at the repo root
- Xcode project with SPM for any dependencies

## Technology Choices

| Concern | Choice | Rationale |
|---|---|---|
| UI | SwiftUI | Native, lean, declarative |
| Networking | URLSession | Built-in, async/await. Zero dependencies. |
| Local Storage | UserDefaults + Keychain | URL in UserDefaults; API key in Keychain. No database needed on client. |
| Push | APNs with token-based auth (.p8) | No cert renewal, one key for all apps under the team |
| Architecture | MVVM with `@Observable` | Simple, iOS 17 native |
| Dependencies | Zero | Everything needed is in the iOS SDK |

## App Structure

```
ios/Claudiator/
├── ClaudiatorApp.swift
├── Models/
│   ├── Device.swift
│   ├── Session.swift
│   └── Event.swift
├── Services/
│   ├── APIClient.swift            # URLSession wrapper
│   ├── KeychainService.swift      # API key storage
│   └── NotificationService.swift  # APNs registration
├── Views/
│   ├── SetupView.swift            # Server URL + API key onboarding
│   ├── DeviceListView.swift
│   ├── SessionListView.swift
│   └── EventListView.swift
└── Info.plist
```

## Onboarding & Configuration

1. First launch shows **SetupView** — two fields: Server URL, API Key
2. Validates via `GET /api/v1/ping` with provided credentials
3. On success: URL → UserDefaults, API key → Keychain
4. Settings screen available later to change or disconnect

## Push Notifications

### Flow

```
Claude Code hook event
        │
        ▼
  Claudiator Server (POST /api/v1/events)
        │  notifiable event? (Stop, Notification)
        ▼
  APNs (api.push.apple.com via HTTP/2)
        │
        ▼
  iOS App (banner/alert)
```

### Client Side (iOS App)

- Requests push permission after onboarding
- iOS provides a **device token** (unique per app+device)
- App sends token to server: `POST /api/v1/devices/register`
- Re-registers on every launch (token can change)

### Server Side (Claudiator Server)

- Stores APNs device token alongside the API key
- Authenticates to APNs using **token-based JWT** signed with the `.p8` key + Key ID + Team ID
- Sends HTTP/2 POST to `api.push.apple.com` with device token and payload

### Notification Payload

```json
{
  "aps": {
    "alert": {
      "title": "Claude needs input",
      "body": "Permission prompt on shahads-macbook"
    },
    "sound": "default"
  },
  "event_type": "Notification",
  "device_name": "shahads-macbook",
  "session_id": "abc123"
}
```

### Apple Developer Setup Required

1. Register **App ID** with push notification capability enabled
2. Generate **APNs Auth Key** (.p8 file) — once, never expires
3. Pick a **Bundle ID** (e.g. `com.claudiator.app`)
4. Xcode automatic signing handles provisioning

### Server Changes Required

1. **New endpoint**: `POST /api/v1/devices/register` — accepts `{ "platform": "ios", "push_token": "..." }`
2. **New DB column**: `push_token` on `devices` table (nullable)
3. **APNs HTTP/2 client** for sending pushes
4. **Event routing**: notifiable event → look up device tokens → send push
5. **New env vars**: `CLAUDIATOR_APNS_KEY_PATH`, `CLAUDIATOR_APNS_KEY_ID`, `CLAUDIATOR_APNS_TEAM_ID`, `CLAUDIATOR_APNS_BUNDLE_ID`

## Distribution

- **TestFlight** for initial rollout — archive in Xcode, upload to App Store Connect, distribute
- **App Store** when ready — needs privacy policy, working server during review
- Self-hosted/configurable apps pass review fine (Nextcloud, Home Assistant pattern)

## Open Questions

- **Badge management**: Track unread count on server and set badge numbers, or skip badges for v1?
- **Notification grouping**: Group by device? By session? Flat list?
- **In-app real-time**: Simple polling (10-30s) for v1, SSE later if needed?
