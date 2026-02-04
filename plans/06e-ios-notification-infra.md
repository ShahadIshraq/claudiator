# Phase 4e — iOS: Notification Infrastructure ✅

**Status**: COMPLETED

## Overview

Add the core notification infrastructure to the iOS app: data model, API methods, NotificationManager service, and VersionMonitor integration. This enables the polling-based notification path.

## Completed Features

✅ All planned features implemented
✅ Additional enhancements added:
- APNs push/polling deduplication with 1-minute retention
- Immediate poll trigger when APNs push received
- content-available flag for background notification handling
- Made optional fields in AppNotification for server compatibility
- Fixed URL construction bug (appendingPathComponent encoding issue)

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

## Additional Implementation Details

### Deduplication System
NotificationManager tracks push-received notification IDs to prevent duplicates:
- `markReceivedViaPush(notificationId:)` - Called when APNs push arrives
- `pushReceivedRetentionSeconds: 60` - 1-minute cleanup window
- Automatic cleanup of expired entries via timestamps

### Immediate Poll Trigger
AppDelegate's `didReceiveRemoteNotification`:
- Marks notification as received via push
- Immediately triggers `fetchNewNotifications()` for instant UI update
- No need to wait for 10-second polling interval

### Server Payload Enhancement
APNs payload includes custom fields for deduplication:
```json
{
  "aps": { "alert": {...}, "content-available": 1 },
  "notification_id": "uuid",
  "session_id": "...",
  "device_id": "..."
}
```

## Files Modified

| File | Action |
|---|---|
| `ios/Claudiator/Models/AppNotification.swift` | **create** |
| `ios/Claudiator/Services/NotificationManager.swift` | **create** |
| `ios/Claudiator/Services/APIClient.swift` | modify — ping return type + 2 methods |
| `ios/Claudiator/Services/VersionMonitor.swift` | modify — track notificationVersion, accept notificationManager |
| `ios/Claudiator/ClaudiatorApp.swift` | modify — register NotificationManager in environment + APNs handling |
| `server/src/apns.rs` | modify — add custom fields to push payload |
| `server/src/handlers/events.rs` | modify — pass notification metadata to APNs |
