# APNs Setup Guide

This guide walks you through enabling push notifications for Claudiator via Apple Push Notification service (APNs).

## 1. Create an APNs Key

1. Go to [Apple Developer > Keys](https://developer.apple.com/account/resources/authkeys/list)
2. Click **+** to create a new key
3. Name it (e.g., "Claudiator APNs"), check **Apple Push Notifications service (APNs)**
4. Click **Continue**, then **Register**
5. **Download the `.p8` file** — you can only download it once
6. Note the **Key ID** (10-character string shown on the key details page)

## 2. Find Your IDs

- **Team ID** — [Account > Membership Details](https://developer.apple.com/account/#/membership/), listed as "Team ID"
- **Bundle ID** — The bundle identifier of your iOS app (e.g., `com.yourname.claudiator`)

## 3. Configure the Server

Set all four APNs environment variables. If any are missing, APNs is disabled and the server runs without push notifications.

```bash
export CLAUDIATOR_APNS_KEY_PATH=/path/to/AuthKey_XXXXXXXXXX.p8
export CLAUDIATOR_APNS_KEY_ID=XXXXXXXXXX
export CLAUDIATOR_APNS_TEAM_ID=YYYYYYYYYY
export CLAUDIATOR_APNS_BUNDLE_ID=com.yourname.claudiator
export CLAUDIATOR_APNS_SANDBOX=true   # false for App Store builds
```

Or pass as CLI flags:

```bash
claudiator-server --api-key <key> \
  --apns-key-path /path/to/AuthKey.p8 \
  --apns-key-id XXXXXXXXXX \
  --apns-team-id YYYYYYYYYY \
  --apns-bundle-id com.yourname.claudiator \
  --apns-sandbox
```

For production deployments, add these to your `/opt/claudiator/.env` file.

## 4. Sandbox vs Production

| Build type | APNs endpoint | `APNS_SANDBOX` |
|---|---|---|
| Xcode debug / Simulator | `api.sandbox.push.apple.com` | `true` |
| TestFlight | `api.sandbox.push.apple.com` | `true` |
| App Store | `api.push.apple.com` | `false` |

## 5. Verify

Start the server and check logs. On success you'll see:

```
APNs client initialized (sandbox: true)
```

If APNs config is incomplete, you'll see:

```
APNs not configured — push notifications disabled
```

## 6. How It Works

1. The iOS app registers its device token via `POST /api/v1/push/register`
2. When a qualifying hook event arrives (`Stop`, `permission_prompt`, `idle_prompt`), the server creates a notification record and sends an APNs push to all registered tokens
3. Stale tokens (APNs `410 Gone` response) are automatically removed
4. Notifications expire after 24 hours

See [API.md](API.md) for endpoint details.
