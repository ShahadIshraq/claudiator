# Phase 4e — iOS: Notification Infrastructure

## Overview

Add the core notification infrastructure to the iOS app: data model, API methods, NotificationManager service, and VersionMonitor integration. This enables the polling-based notification path.

## AppNotification Model

New file: `ios/Claudiator/Models/AppNotification.swift`

```swift
struct AppNotification: Codable, Identifiable, Hashable {
    var id: String { notificationId }
    let notificationId: String  // maps from "id" in JSON
    let sessionId: String
    let deviceId: String
    let title: String
    let body: String
    let notificationType: String
    let payloadJson: String
    let createdAt: String
    let acknowledged: Bool

    enum CodingKeys: String, CodingKey {
        case notificationId = "id"
        case sessionId, deviceId, title, body
        case notificationType, payloadJson, createdAt, acknowledged
    }
}
```

Named `AppNotification` to avoid collision with the system `Notification` type.

## APIClient Updates

`ios/Claudiator/Services/APIClient.swift`:

### ping() — breaking change

Change return type from `UInt64` to `(dataVersion: UInt64, notificationVersion: UInt64)`:

```swift
func ping() async throws -> (dataVersion: UInt64, notificationVersion: UInt64) {
    let data = try await request("/api/v1/ping")
    struct PingResponse: Decodable {
        let status: String
        let serverVersion: String?
        let dataVersion: UInt64?
        let notificationVersion: UInt64?
    }
    let response = try Self.decoder.decode(PingResponse.self, from: data)
    return (response.dataVersion ?? 0, response.notificationVersion ?? 0)
}
```

### New methods

```swift
func fetchNotifications(since: String? = nil, limit: Int? = nil) async throws -> [AppNotification] {
    var path = "/api/v1/notifications"
    var params: [String] = []
    if let since { params.append("since=\(since)") }
    if let limit { params.append("limit=\(limit)") }
    if !params.isEmpty { path += "?" + params.joined(separator: "&") }
    let data = try await request(path)
    struct Wrapper: Decodable { let notifications: [AppNotification] }
    return try Self.decoder.decode(Wrapper.self, from: data).notifications
}

func acknowledgeNotifications(ids: [String]) async throws {
    let body = try JSONEncoder().encode(["notification_ids": ids])
    _ = try await request("/api/v1/notifications/ack", method: "POST", body: body)
}
```

## NotificationManager Service

New file: `ios/Claudiator/Services/NotificationManager.swift`

`@Observable` class with:

- `unreadNotifications: [AppNotification]`
- `allNotifications: [AppNotification]`
- `unreadCount: Int`
- `sessionsWithNotifications: Set<String>` — for session row highlighting

### Key methods

- `fetchNewNotifications(apiClient:)` — fetches since last-seen UUID, fires local `UNNotificationRequest` for each (UUID as identifier for iOS dedup), updates internal state
- `markSessionRead(sessionId)` — removes all notifications for that session from unread set
- `markNotificationRead(notificationId)` — removes single notification from unread set

### Persistence

- `lastSeenNotificationId` in UserDefaults — tracks cursor for pagination
- `readNotificationIds: Set<String>` in UserDefaults — tracks which notifications user has seen
- Cap `allNotifications` at 100 entries

### Local notification firing

```swift
private func fireLocalNotification(_ notif: AppNotification) async {
    let content = UNMutableNotificationContent()
    content.title = notif.title
    content.body = notif.body
    content.sound = .default
    content.userInfo = [
        "notification_id": notif.notificationId,
        "session_id": notif.sessionId,
        "device_id": notif.deviceId,
    ]

    let request = UNNotificationRequest(
        identifier: notif.notificationId,  // iOS deduplicates by this
        content: content,
        trigger: nil  // Deliver immediately
    )

    try? await UNUserNotificationCenter.current().add(request)
}
```

## VersionMonitor Update

`ios/Claudiator/Services/VersionMonitor.swift`:

- Add `notificationVersion: UInt64` property
- Change `start()` to accept `notificationManager` parameter
- When `notificationVersion` changes, call `notificationManager.fetchNewNotifications()`

```swift
func start(apiClient: APIClient, notificationManager: NotificationManager) {
    guard task == nil else { return }
    task = Task {
        while !Task.isCancelled {
            if let versions = try? await apiClient.ping() {
                let oldNotifVersion = self.notificationVersion
                await MainActor.run {
                    self.dataVersion = versions.dataVersion
                    self.notificationVersion = versions.notificationVersion
                }
                if versions.notificationVersion != oldNotifVersion {
                    await notificationManager.fetchNewNotifications(apiClient: apiClient)
                }
            }
            try? await Task.sleep(for: .seconds(10))
        }
    }
}
```

## App Entry Point

`ios/Claudiator/ClaudiatorApp.swift`:

- Add `@State private var notificationManager = NotificationManager()`
- Pass into environment: `.environment(notificationManager)`
- Update `versionMonitor.start()` call in MainTabView to include `notificationManager`

## Files Modified

| File | Action |
|---|---|
| `ios/Claudiator/Models/AppNotification.swift` | **create** |
| `ios/Claudiator/Services/NotificationManager.swift` | **create** |
| `ios/Claudiator/Services/APIClient.swift` | modify — ping return type + 2 methods |
| `ios/Claudiator/Services/VersionMonitor.swift` | modify — track notificationVersion, accept notificationManager |
| `ios/Claudiator/ClaudiatorApp.swift` | modify — register NotificationManager in environment |
