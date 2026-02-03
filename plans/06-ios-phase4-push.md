# Phase 4 — Push Notifications

Two sides: iOS client registration and server-side APNs dispatch.

## Apple Developer Portal Setup

1. In Certificates, Identifiers & Profiles → **Keys** → create a new key
2. Enable **Apple Push Notifications service (APNs)**
3. Download the `.p8` file — save it securely, it can only be downloaded once
4. Note the **Key ID** (10-char string shown in the portal)
5. Note your **Team ID** (from Membership details)
6. Ensure the App ID (`com.claudiator.app`) has Push Notifications capability enabled

## iOS Client Side

### Requesting Permission

On successful onboarding (after SetupView), request notification permission:

```swift
func requestPushPermission() async {
    let center = UNUserNotificationCenter.current()
    let granted = try? await center.requestAuthorization(options: [.alert, .sound, .badge])
    if granted == true {
        await MainActor.run {
            UIApplication.shared.registerForRemoteNotifications()
        }
    }
}
```

### Receiving the Device Token

In `ClaudiatorApp`, use `UIApplicationDelegateAdaptor` to capture the token:

```swift
class AppDelegate: NSObject, UIApplicationDelegate {
    func application(_ app: UIApplication, didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
        let token = deviceToken.map { String(format: "%02x", $0) }.joined()
        Task {
            try? await apiClient.registerPushToken(platform: "ios", token: token)
        }
    }

    func application(_ app: UIApplication, didFailToRegisterForRemoteNotificationsWithError error: Error) {
        // Log, don't crash. Push is optional.
    }
}
```

### Token Registration

- Call `POST /api/v1/push/register` with the hex-encoded token
- Re-register on every app launch (token can change)
- If server returns 401, redirect to SetupView

### Handling Incoming Notifications

For foreground notifications, implement `UNUserNotificationCenterDelegate`:

```swift
func userNotificationCenter(_ center: UNUserNotificationCenter,
                            willPresent notification: UNNotification) async -> UNNotificationPresentationOptions {
    return [.banner, .sound]  // Show banner even when app is in foreground
}
```

Tapping a notification could deep-link to the relevant session (using `session_id` from the payload's custom data).

## Server Side — APNs Integration

### New Dependencies (Cargo.toml)

```toml
jsonwebtoken = "9"          # JWT signing for APNs auth
hyper = { version = "1", features = ["client", "http2"] }  # HTTP/2 client for APNs
hyper-rustls = "0.27"       # TLS for APNs connection
```

Alternatively, use the `a2` crate which wraps APNs HTTP/2 communication. Evaluate size/complexity tradeoff.

### Server Configuration

New env vars added to `.env`:

```
CLAUDIATOR_APNS_ENABLED=true
CLAUDIATOR_APNS_KEY_PATH=/opt/claudiator/apns-key.p8
CLAUDIATOR_APNS_KEY_ID=ABC123DEF4
CLAUDIATOR_APNS_TEAM_ID=TEAM123456
CLAUDIATOR_APNS_BUNDLE_ID=com.claudiator.app
CLAUDIATOR_APNS_SANDBOX=false
```

`APNS_SANDBOX=true` uses `api.sandbox.push.apple.com` for development/TestFlight builds. Production uses `api.push.apple.com`.

### APNs JWT Authentication

The server signs a short-lived JWT (valid for up to 1 hour, reusable across requests):

**Header:**
```json
{
  "alg": "ES256",
  "kid": "{APNS_KEY_ID}"
}
```

**Claims:**
```json
{
  "iss": "{APNS_TEAM_ID}",
  "iat": 1706900000
}
```

Sign with the `.p8` private key (ES256 / P-256 ECDSA). Cache the JWT and refresh every 50 minutes.

### Sending a Push

HTTP/2 POST to `https://api.push.apple.com/3/device/{device_token}`:

**Headers:**
```
authorization: bearer {jwt}
apns-topic: com.claudiator.app
apns-push-type: alert
apns-priority: 10
```

**Body:**
```json
{
  "aps": {
    "alert": {
      "title": "Claude needs input",
      "body": "Permission prompt on shahads-macbook"
    },
    "sound": "default"
  },
  "session_id": "abc123",
  "device_name": "shahads-macbook",
  "event_type": "Notification"
}
```

### Event-to-Notification Routing

Add to the existing event ingestion handler (`handlers/events.rs`), after the DB transaction commits:

```
if hook_event_name is "Stop" or "Notification":
    1. Look up all push_tokens where platform = "ios"
    2. Build notification payload (title/body based on event type)
    3. Send push to each token (fire-and-forget, don't block the event response)
    4. If APNs returns 410 (Unregistered), delete the token from push_tokens
```

### Notification Content by Event Type

| Event | Title | Body |
|---|---|---|
| Stop | "Claude is waiting" | "Session idle on {device_name}" |
| Notification (permission_prompt) | "Permission needed" | "{tool_name} on {device_name}" |
| Notification (idle_prompt) | "Session idle" | "Idle on {device_name}" |
| Notification (other) | "Claude notification" | "{message}" or "Event on {device_name}" |

### APNs Error Handling

- **200**: Success
- **400**: Bad request (malformed payload) — log and skip
- **403**: Auth error (bad JWT) — refresh JWT and retry once
- **410**: Device token unregistered — delete from `push_tokens` table
- **429**: Too many requests — back off
- **503**: Service unavailable — retry with backoff

Push failures should never block or fail the event ingestion response. Log errors, move on.
