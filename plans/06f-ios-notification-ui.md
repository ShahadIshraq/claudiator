# Phase 4f — iOS: Notification UI ✅

**Status**: COMPLETED

## Overview

Add notification-related UI to the iOS app: bell icon with badge count, notification list sheet, session card highlighting for unread notifications, and foreground notification banners.

## Completed Features

✅ All planned features implemented
✅ Enhanced with pulsing indicators:
- Session cards pulse brightness (4-12%) when containing unread notifications
- Group containers pulse opacity (30-70%) when containing unread notifications
- Smooth 1.2s continuous animation cycle
- Theme-aware adaptive colors for all themes
- Swipe-to-mark-read gesture in notification list

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

## Additional Implementation Details

### Pulsing Notification Indicators

Instead of static highlighting, implemented breathing animations:

**Session Cards**:
- Normal: Base brightness
- With notifications: Pulse between +4% and +12% brightness
- Helper function: `sessionCardBrightness(_:) -> Double`

**Group Containers**:
- Normal: 30% opacity
- With notifications: Pulse between 50% and 70% opacity
- Helper function: `groupContainerOpacity(_:) -> Double`

**Animation System**:
```swift
@State private var notificationPulse: Bool = false

.onAppear {
    Task {
        while !Task.isCancelled {
            try? await Task.sleep(for: .seconds(1.2))
            notificationPulse.toggle()
        }
    }
}
```

### Theme-Aware Colors

Made `eventNotification` adaptive across all themes:
- Standard: Adaptive red (light/dark variants)
- Neon Ops: Adaptive pink (#FF4080)
- Solarized: Adaptive red (#DC322F)
- Arctic: Adaptive red (#EF4444)

All colors now properly respond to theme changes and light/dark mode.

### Notification List Enhancements

Beyond planned features:
- Swipe-to-mark-read gesture with blue checkmark button
- Full-swipe support for quick dismissal
- Read/unread visual distinction maintained

## Files Modified

| File | Action |
|---|---|
| `ios/Claudiator/Views/NotificationListView.swift` | **create** — with swipe gestures |
| `ios/Claudiator/Views/AllSessionsView.swift` | modify — bell icon + pulsing indicators + helpers |
| `ios/Claudiator/Views/SessionDetailView.swift` | modify — markSessionRead in .task |
| `ios/Claudiator/ClaudiatorApp.swift` | modify — UNUserNotificationCenterDelegate + permission |
| `ios/Claudiator/Theme/AppTheme+Themes.swift` | modify — adaptive notification colors |
