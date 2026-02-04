# Phase 4f — iOS: Notification UI

## Overview

Add notification-related UI to the iOS app: bell icon with badge count, notification list sheet, session card highlighting for unread notifications, and foreground notification banners.

## Bell Icon + Notification List

`ios/Claudiator/Views/AllSessionsView.swift`:

- Add `@Environment(NotificationManager.self) private var notificationManager`
- Add `@State private var showNotifications = false`

### Toolbar bell icon

In `.toolbar`, add a leading item:

```swift
ToolbarItem(placement: .navigationBarLeading) {
    Button {
        showNotifications = true
    } label: {
        Image(systemName: "bell")
            .overlay(alignment: .topTrailing) {
                if notificationManager.unreadCount > 0 {
                    Text("\(notificationManager.unreadCount)")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundStyle(.white)
                        .padding(3)
                        .background(themeManager.current.uiError)
                        .clipShape(Circle())
                        .offset(x: 6, y: -6)
                }
            }
    }
}
```

Add sheet:

```swift
.sheet(isPresented: $showNotifications) {
    NotificationListView()
}
```

## Notification List View

New file: `ios/Claudiator/Views/NotificationListView.swift`

- Navigation stack with "Notifications" title and "Done" dismiss button
- List of `AppNotification` items in reverse chronological order
- Empty state: `ContentUnavailableView` with bell.slash icon

### NotificationRow

Each row shows:
- Unread dot indicator (8px circle, themed tint color when unread, clear when read)
- Icon per notification type:
  - `permission_prompt` → `lock.shield`
  - `idle_prompt` → `moon.zzz`
  - `stop` → `hand.raised`
  - default → `bell`
- Title (subheadline, medium weight)
- Body (caption, secondary color)
- Relative time (caption2, secondary color)

Tapping marks the notification as read.

## Session Card Highlight

`ios/Claudiator/Views/AllSessionsView.swift` — `AllSessionRow`:

- Add `@Environment(NotificationManager.self) private var notificationManager`
- Check `notificationManager.sessionsWithNotifications.contains(session.sessionId)`
- If true: show colored left border (3px `eventNotification` color)

```swift
.overlay(alignment: .leading) {
    if hasNotification {
        RoundedRectangle(cornerRadius: 2)
            .fill(themeManager.current.eventNotification)
            .frame(width: 3)
    }
}
```

## Clear Notification on Session View

`ios/Claudiator/Views/SessionDetailView.swift`:

- Add `@Environment(NotificationManager.self) private var notificationManager`
- In `.task`: call `notificationManager.markSessionRead(session.sessionId)`

## Foreground Notification Handling

Set up `UNUserNotificationCenterDelegate` to show banners when app is in foreground.

In `ClaudiatorApp.swift` or via an AppDelegate adaptor:

```swift
func userNotificationCenter(_ center: UNUserNotificationCenter,
                            willPresent notification: UNNotification) async -> UNNotificationPresentationOptions {
    return [.banner, .sound]
}
```

## Notification Permission

Request local notification permission after setup completes (in `SetupView` or after `SetupViewModel` succeeds):

```swift
let center = UNUserNotificationCenter.current()
_ = try? await center.requestAuthorization(options: [.alert, .sound, .badge])
```

## Files Modified

| File | Action |
|---|---|
| `ios/Claudiator/Views/NotificationListView.swift` | **create** |
| `ios/Claudiator/Views/AllSessionsView.swift` | modify — bell icon in toolbar + session row highlight |
| `ios/Claudiator/Views/SessionDetailView.swift` | modify — markSessionRead in .task |
| `ios/Claudiator/ClaudiatorApp.swift` | modify — UNUserNotificationCenterDelegate + permission request |
