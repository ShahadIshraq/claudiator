# Phase 3 — iOS Main Views

## Navigation Structure

```
DeviceListView
    └── SessionListView (for selected device)
            └── EventListView (for selected session)
```

Standard `NavigationStack` with push navigation.

## DeviceListView

Displays all Claude Code machines reporting to the server.

- Calls `GET /api/v1/devices` on appear
- Pull-to-refresh
- Auto-refresh via polling (every 15s)
- Each row shows:
  - Device name (bold)
  - Platform icon (laptop for mac/linux, desktop for windows)
  - Last seen (relative time: "2 min ago", "1 hour ago")
  - Active session count badge
- Tapping a device navigates to `SessionListView`
- Empty state: "No devices found" with explanation

## SessionListView

Displays sessions for a selected device.

- Calls `GET /api/v1/devices/:device_id/sessions` on appear
- Pull-to-refresh
- Auto-refresh via polling (every 10s)
- Each row shows:
  - Session status indicator (colored dot):
    - Green: `active`
    - Orange: `waiting_for_input`
    - Red: `waiting_for_permission`
    - Gray: `ended` / `idle`
  - Working directory (`cwd`), truncated to last 2 path components
  - Last event time (relative)
  - Status label text
- Tapping a session navigates to `EventListView`
- Segmented control or filter: Active / All

## EventListView

Displays events for a selected session in reverse chronological order.

- Calls `GET /api/v1/sessions/:session_id/events` on appear
- Pull-to-refresh
- Auto-refresh via polling (every 10s)
- Each row shows:
  - Event type icon/label (SessionStart, Stop, Notification, etc.)
  - Timestamp (relative)
  - Message or tool name if present
  - Notification type if present
- Color coding matches session status indicators

## Shared Patterns

### Polling

Simple `Timer` or `.task` with `try await Task.sleep` loop. Cancel on view disappear.

```swift
.task {
    while !Task.isCancelled {
        await viewModel.refresh()
        try? await Task.sleep(for: .seconds(15))
    }
}
```

### Error Handling

- Network errors → show inline banner at top ("Connection lost"), auto-retry on next poll
- 401 errors → redirect to SetupView (API key invalid/changed)
- Loading states → `ProgressView` on first load, silent refresh on subsequent polls

### Relative Time Formatting

Use `RelativeDateTimeFormatter` for "2 min ago" style strings. Parse RFC 3339 timestamps with `ISO8601DateFormatter`.

### iPad Layout

- `DeviceListView` as sidebar in `NavigationSplitView` on iPad
- Sessions and events in detail pane
- Falls back to stack navigation on iPhone
- Use `@Environment(\.horizontalSizeClass)` to adapt

```swift
var body: some Scene {
    WindowGroup {
        if isConfigured {
            NavigationSplitView {
                DeviceListView()
            } detail: {
                Text("Select a device")
            }
        } else {
            SetupView(isConfigured: $isConfigured)
        }
    }
}
```
