# Phase 4 — Hybrid Notifications (Overview)

Three delivery tiers sharing one dedup key (notification UUID):

1. **Polling (free, app open)**: App detects `notification_version` change via 10s ping, fetches notifications, fires local `UNNotificationRequest`
2. **Direct APNs (self-hosted with key)**: Server dispatches APNs push directly if `.p8` key is configured — optional during install
3. **APNs proxy (future paid)**: Self-hosted servers without a key POST to `push.claudiator.com` for real-time push

This design lets the app ship as free/self-hosted. APNs proxy becomes a paid add-on later — zero iOS code changes needed since all paths use the same UUID for deduplication.

## Sub-phases

| File | Phase | Description |
|---|---|---|
| [06a-server-notification-table.md](06a-server-notification-table.md) | 4a | Notifications table, `should_notify()` logic, event handler integration |
| [06b-server-ping-and-endpoints.md](06b-server-ping-and-endpoints.md) | 4b | `notification_version` in ping, GET/POST notification endpoints |
| [06c-server-apns-direct.md](06c-server-apns-direct.md) | 4c | Optional APNs client, JWT auth, HTTP/2 dispatch |
| [06d-server-install-script.md](06d-server-install-script.md) | 4d | Install script APNs prompts, conditional .env vars |
| [06e-ios-notification-infra.md](06e-ios-notification-infra.md) | 4e | AppNotification model, NotificationManager, APIClient + VersionMonitor updates |
| [06f-ios-notification-ui.md](06f-ios-notification-ui.md) | 4f | Bell icon, notification list, session card highlight, foreground banners |

## Deduplication Strategy

- Notification UUID is the single dedup key across all delivery paths
- iOS uses UUID as `UNNotificationRequest.identifier` — iOS itself prevents duplicate display
- APNs uses UUID as `apns-collapse-id` — Apple deduplicates server-side
- If both polling and APNs deliver the same notification, only one banner appears

## Parallelization

- **Agent A**: Server 4a + 4b + 4c (notification table, endpoints, APNs client)
- **Agent B**: Server 4d (install script) — can run with A
- **Agent C**: iOS 4e (models, services, APIClient, VersionMonitor)
- **Agent D**: iOS 4f (views, bell icon, highlighting) — depends on C

A + B + C can run in parallel. D runs after C.

## Future: APNs Push Proxy (Paid Tier)

Design only — no implementation in this phase.

Self-hosted servers call:
```
POST https://push.claudiator.com/api/v1/dispatch
Authorization: Bearer <subscription_token>

{
  "notification_id": "uuid",
  "push_token": "hex-device-token",
  "title": "Claude is waiting",
  "body": "Session idle on shahads-macbook",
  "payload": { "session_id": "abc123", "device_id": "..." }
}
```

The proxy validates subscription, signs JWT with our APNs key, dispatches via HTTP/2 to APNs, uses notification UUID as `apns-collapse-id`. Zero iOS app changes needed.
