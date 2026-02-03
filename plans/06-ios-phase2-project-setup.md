# Phase 2 — iOS Project Setup & Onboarding

## Xcode Project

- Create `ios/` directory at repo root
- New Xcode project: **Claudiator**, bundle ID `com.claudiator.app`
- Deployment target: iOS 17.0
- Devices: iPhone + iPad (universal)
- SwiftUI lifecycle (`@main` App struct)
- Enable Push Notifications capability in Signing & Capabilities
- Enable Background Modes → Remote notifications

## Project Structure

```
ios/Claudiator/
├── ClaudiatorApp.swift
├── Models/
│   ├── Device.swift
│   ├── Session.swift
│   └── Event.swift
├── Services/
│   ├── APIClient.swift
│   ├── KeychainService.swift
│   └── NotificationService.swift
├── ViewModels/
│   ├── SetupViewModel.swift
│   ├── DeviceListViewModel.swift
│   ├── SessionListViewModel.swift
│   └── EventListViewModel.swift
├── Views/
│   ├── SetupView.swift
│   ├── DeviceListView.swift
│   ├── SessionListView.swift
│   └── EventListView.swift
└── Info.plist
```

## Models

Decodable structs matching the server API responses from Phase 1:

```swift
struct Device: Codable, Identifiable {
    var id: String { deviceId }
    let deviceId: String
    let deviceName: String
    let platform: String
    let firstSeen: String
    let lastSeen: String
    let activeSessions: Int
}

struct Session: Codable, Identifiable {
    var id: String { sessionId }
    let sessionId: String
    let deviceId: String
    let startedAt: String
    let lastEvent: String
    let status: String
    let cwd: String?
}

struct Event: Codable, Identifiable {
    let id: Int
    let hookEventName: String
    let timestamp: String
    let toolName: String?
    let notificationType: String?
    let message: String?
}
```

Use `JSONDecoder` with `.convertFromSnakeCase` key strategy.

## APIClient

Single class wrapping URLSession for all server communication:

```swift
@Observable
class APIClient {
    var baseURL: URL?
    var apiKey: String?

    func ping() async throws -> Bool
    func fetchDevices() async throws -> [Device]
    func fetchSessions(deviceId: String, status: String?, limit: Int?) async throws -> [Session]
    func fetchEvents(sessionId: String, limit: Int?) async throws -> [Event]
    func registerPushToken(platform: String, token: String) async throws
}
```

- All requests add `Authorization: Bearer {apiKey}` header
- All requests use `Content-Type: application/json`
- Throw typed errors for network failure, auth failure (401), server error

## KeychainService

Minimal wrapper around Security framework for storing/retrieving the API key:

```swift
enum KeychainService {
    static func save(key: String, value: String) throws
    static func load(key: String) throws -> String?
    static func delete(key: String) throws
}
```

Uses `kSecClassGenericPassword` with service name `com.claudiator.app`.

## Onboarding Flow (SetupView)

1. App launch checks: is there a saved server URL (UserDefaults) and API key (Keychain)?
2. If no → show `SetupView`
3. `SetupView` has:
   - Text field for server URL (keyboard type `.URL`, autocapitalization off)
   - Secure text field for API key
   - "Connect" button
4. On tap "Connect":
   - Validate URL format
   - Call `GET /api/v1/ping` with provided credentials
   - On success → save URL to UserDefaults, API key to Keychain → navigate to main view
   - On failure → show inline error ("Connection failed", "Invalid API key", etc.)
5. Main view shows a settings/gear icon to return to setup and change config

## App Entry Point

```swift
@main
struct ClaudiatorApp: App {
    @State private var isConfigured = false

    var body: some Scene {
        WindowGroup {
            if isConfigured {
                DeviceListView()
            } else {
                SetupView(isConfigured: $isConfigured)
            }
        }
    }
}
```

Check config on appear — if URL and API key exist, set `isConfigured = true`.
