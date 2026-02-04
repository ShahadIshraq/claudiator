# iOS Notification Implementation Status & Next Steps

## Current State Assessment (as of Feb 4, 2026)

### Server Infrastructure ✅ 100% COMPLETE

The server has all required notification infrastructure:

**Completed:**
- ✅ `notifications` table with UUID primary key
- ✅ Notification generation on `Stop` and `Notification` events
- ✅ `notification_version` counter in AppState
- ✅ `GET /api/v1/ping` returns both `dataVersion` and `notificationVersion`
- ✅ `GET /api/v1/notifications?since=<uuid>&limit=<n>` endpoint
- ✅ `POST /api/v1/notifications/ack` endpoint
- ✅ `POST /api/v1/push/register` endpoint
- ✅ APNs client module (`server/src/apns.rs`) with JWT signing
- ✅ APNs push dispatch in background after notification creation
- ✅ Per-token sandbox flag routing

**Latest commits:**
- `c33f17c` - Update default bundle ID and add ios/.env to gitignore
- `0cb97f4` - Wire up push notification token registration
- `2453283` - Add notification pipeline and APNs integration to events handler
- `79d2a8c` - Add notifications endpoint and version tracking

### iOS App Status ⚠️ 40% COMPLETE

**What exists (✅):**
- ✅ Push token registration (`NotificationService.swift`, `APIClient.registerPushToken()`)
- ✅ AppDelegate wiring for remote notifications
- ✅ Basic app structure (tabs, theme system, settings, session views)

**What's MISSING - Phase 4e (iOS Notification Infrastructure):**

The polling-based notification system is **not implemented**. This is the core functionality that makes notifications work:

❌ **1. AppNotification Model** (`ios/Claudiator/Models/AppNotification.swift`)
   - Not created
   - See `06e-ios-notification-infra.md` lines 7-30 for spec

❌ **2. APIClient Updates** (`ios/Claudiator/Services/APIClient.swift`)
   - Current: `ping()` returns `UInt64` (line 72-81)
   - **Need:** Return `(dataVersion: UInt64, notificationVersion: UInt64)` tuple
   - **Need:** Add `fetchNotifications(since:limit:)` method
   - **Need:** Add `acknowledgeNotifications(ids:)` method
   - See `06e-ios-notification-infra.md` lines 34-74 for spec

❌ **3. NotificationManager Service** (`ios/Claudiator/Services/NotificationManager.swift`)
   - Not created
   - This is the core service that:
     - Fetches notifications from server
     - Fires local `UNNotificationRequest` per notification (UUID as identifier)
     - Maintains unread state
     - Provides `sessionsWithNotifications` for UI highlighting
   - See `06e-ios-notification-infra.md` lines 76-121 for spec

❌ **4. VersionMonitor Updates** (`ios/Claudiator/Services/VersionMonitor.swift`)
   - Current: Only tracks `dataVersion` (line 6)
   - **Need:** Add `notificationVersion` property
   - **Need:** Accept `notificationManager` parameter in `start()`
   - **Need:** Call `notificationManager.fetchNewNotifications()` when version changes
   - See `06e-ios-notification-infra.md` lines 123-150 for spec

❌ **5. ClaudiatorApp Wiring** (`ios/Claudiator/ClaudiatorApp.swift`)
   - **Need:** Create `NotificationManager` instance
   - **Need:** Pass to environment
   - **Need:** Update `versionMonitor.start()` call to include notificationManager
   - See `06e-ios-notification-infra.md` lines 152-158

**What's MISSING - Phase 4f (iOS Notification UI):**

Once Phase 4e is complete, add UI:

❌ **1. NotificationListView** (`ios/Claudiator/Views/NotificationListView.swift`)
   - Full notification list sheet
   - See `06f-ios-notification-ui.md` lines 47-69

❌ **2. Bell Icon** (`ios/Claudiator/Views/AllSessionsView.swift`)
   - Toolbar leading item with badge count
   - Sheet presentation
   - See `06f-ios-notification-ui.md` lines 8-45

❌ **3. Session Card Highlighting** (`ios/Claudiator/Views/AllSessionsView.swift`)
   - Colored left border for sessions with unread notifications
   - See `06f-ios-notification-ui.md` lines 71-86

❌ **4. Clear on Session View** (`ios/Claudiator/Views/SessionDetailView.swift`)
   - Call `markSessionRead()` in `.task`
   - See `06f-ios-notification-ui.md` lines 88-93

❌ **5. Foreground Banner Handling** (`ios/Claudiator/ClaudiatorApp.swift`)
   - `UNUserNotificationCenterDelegate` setup
   - See `06f-ios-notification-ui.md` lines 95-106

## Implementation Plan

### Step 1: Phase 4e - iOS Notification Infrastructure (CRITICAL PATH)

This must be implemented before notifications will work. The server is ready, but iOS can't receive or display notifications yet.

**Create 2 new files:**
1. `ios/Claudiator/Models/AppNotification.swift`
2. `ios/Claudiator/Services/NotificationManager.swift`

**Modify 3 existing files:**
3. `ios/Claudiator/Services/APIClient.swift` - Update ping + add 2 methods
4. `ios/Claudiator/Services/VersionMonitor.swift` - Track notification version
5. `ios/Claudiator/ClaudiatorApp.swift` - Wire up NotificationManager

**Detailed spec:** See `/plans/06e-ios-notification-infra.md`

**Estimated work:** ~2-3 hours for experienced iOS developer

### Step 2: Phase 4f - iOS Notification UI

Add user-facing notification features.

**Create 1 new file:**
1. `ios/Claudiator/Views/NotificationListView.swift`

**Modify 3 existing files:**
2. `ios/Claudiator/Views/AllSessionsView.swift` - Bell icon + session highlighting
3. `ios/Claudiator/Views/SessionDetailView.swift` - Clear on view
4. `ios/Claudiator/ClaudiatorApp.swift` - Foreground delegate

**Detailed spec:** See `/plans/06f-ios-notification-ui.md`

**Estimated work:** ~2-3 hours

### Step 3: Testing

**Test polling notifications:**
1. Run iOS app in simulator or device
2. Trigger a `Stop` event on desktop Claude Code
3. Verify server creates notification (check server logs)
4. Verify iOS detects `notification_version` change within 10 seconds
5. Verify local notification appears
6. Verify bell icon badge updates
7. Tap notification → verify marked as read

**Test APNs push (optional, requires server config):**
1. Configure server with `.p8` key and APNs env vars
2. Run iOS app on physical device
3. Trigger event → verify both APNs push AND polling work
4. Verify UUID deduplication (only one notification shown)

### Step 4: Production Deployment

**Server (already done):**
- Server is deployed and running with notification support
- Optional: Configure APNs for push notifications
- See deployment steps in previous rollout plan if needed

**iOS:**
1. Build release with correct bundle ID
2. Test on physical device
3. Submit to TestFlight for internal testing
4. After validation, submit to App Store

**Documentation:**
- Update README with notification setup
- Document polling vs push notification modes
- User guide for mobile app

## Summary

**Blocker:** iOS notification infrastructure (Phase 4e) is not implemented.

**What works:**
- Server generates notifications ✅
- Server exposes all required endpoints ✅
- iOS can register push tokens ✅ (for future APNs)

**What doesn't work:**
- iOS has no code to poll for notifications ❌
- iOS has no code to display notifications ❌
- iOS has no notification UI ❌

**Next immediate action:**
Implement Phase 4e (iOS Notification Infrastructure) per the detailed spec in `06e-ios-notification-infra.md`. This is ~5 files (2 new, 3 modified) and is the minimum viable implementation to get notifications working.

**Once Phase 4e is done:**
- Notifications will work via polling (free, no APNs config needed)
- Can optionally add Phase 4f UI polish
- Can optionally enable APNs push for instant delivery

## Rollout After iOS Implementation

Once iOS notification code is complete:

### 1. TestFlight Build
- Archive and upload to App Store Connect
- Add to internal testing group
- Install on test devices
- Verify notifications work end-to-end

### 2. APNs Configuration (Optional)
If you want push notifications in addition to polling:

**Server setup:**
```bash
# Upload .p8 key
scp AuthKey.p8 your-server:/opt/claudiator/

# Add to .env
CLAUDIATOR_APNS_KEY_PATH=/opt/claudiator/AuthKey.p8
CLAUDIATOR_APNS_KEY_ID=QARUFK3TXT
CLAUDIATOR_APNS_TEAM_ID=Y4X5LMM4FD
CLAUDIATOR_APNS_BUNDLE_ID=com.claudiator.app
CLAUDIATOR_APNS_SANDBOX=true  # for TestFlight

# Restart server
sudo systemctl restart claudiator-server
```

### 3. App Store Submission
- Add app icon, screenshots, privacy policy
- Submit for review
- After approval: Set `CLAUDIATOR_APNS_SANDBOX=false` for production
