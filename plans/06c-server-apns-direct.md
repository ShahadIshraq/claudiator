# Phase 4c — Server: Optional APNs Direct Push

## Overview

Add optional APNs push dispatch to the server. When configured with an Apple `.p8` key, the server sends push notifications directly to APNs after creating notification records. Without configuration, the server works in polling-only mode.

## Server Config Extension

`server/src/config.rs` — add optional fields:

```rust
#[arg(long, env = "CLAUDIATOR_APNS_KEY_PATH")]
pub apns_key_path: Option<String>,

#[arg(long, env = "CLAUDIATOR_APNS_KEY_ID")]
pub apns_key_id: Option<String>,

#[arg(long, env = "CLAUDIATOR_APNS_TEAM_ID")]
pub apns_team_id: Option<String>,

#[arg(long, default_value = "com.claudiator.app", env = "CLAUDIATOR_APNS_BUNDLE_ID")]
pub apns_bundle_id: String,

#[arg(long, default_value = "false", env = "CLAUDIATOR_APNS_SANDBOX")]
pub apns_sandbox: bool,
```

All APNs fields are `Option` — server works without them.

## AppState Extension

`server/src/router.rs`:

```rust
pub struct AppState {
    pub api_key: String,
    pub db_pool: DbPool,
    pub version: AtomicU64,
    pub notification_version: AtomicU64,
    pub apns_client: Option<Arc<ApnsClient>>,  // None if not configured
}
```

## APNs Client Module

New file: `server/src/apns.rs`

### ApnsClient struct

- Holds: key bytes, key_id, team_id, bundle_id, sandbox flag, cached JWT + expiry (behind a Mutex/RwLock)
- `ApnsClient::new(config) -> Option<Self>` — returns None if key_path/key_id/team_id are not all set. Reads `.p8` file on startup.

### JWT Authentication

- Algorithm: ES256 (P-256 ECDSA)
- Header: `{ "alg": "ES256", "kid": "{key_id}" }`
- Claims: `{ "iss": "{team_id}", "iat": <unix_timestamp> }`
- Sign with `.p8` private key
- Cache JWT, refresh every 50 minutes (APNs allows up to 1 hour)

### Sending a Push

`ApnsClient::send(device_token, notification) -> Result<()>`:

HTTP/2 POST to `https://api.push.apple.com/3/device/{token}` (or `api.sandbox.push.apple.com`):

Headers:
```
authorization: bearer {jwt}
apns-topic: {bundle_id}
apns-push-type: alert
apns-priority: 10
apns-collapse-id: {notification_uuid}
```

Body:
```json
{
  "aps": {
    "alert": { "title": "...", "body": "..." },
    "sound": "default"
  },
  "notification_id": "uuid",
  "session_id": "abc123",
  "device_id": "..."
}
```

### Error Handling

Push failures never block event ingestion:

- **200**: Success
- **400**: Bad request — log and skip
- **403**: Auth error — refresh JWT, retry once
- **410**: Token unregistered — delete via `queries::delete_push_token()`
- **429**: Rate limited — log, backoff
- **503**: Service unavailable — log, backoff

## New Dependencies

`server/Cargo.toml`:

```toml
a2 = "0.10"  # APNs HTTP/2 client (wraps reqwest + h2)
```

Alternative: `jsonwebtoken = "9"` + `reqwest = { features = ["http2"] }` if `a2` is too heavy. Evaluate during implementation.

## Dispatch Integration

`server/src/handlers/events.rs` — after COMMIT, if notification was created AND APNs configured:

```rust
if notification_was_created {
    state.notification_version.fetch_add(1, Ordering::Relaxed);

    if let Some(apns) = &state.apns_client {
        let apns = apns.clone();
        let notification = notification_data.clone();
        let pool = state.db_pool.clone();
        tokio::spawn(async move {
            let conn = pool.get().unwrap();
            let tokens = queries::list_push_tokens(&conn, Some("ios"));
            for token in tokens {
                if let Err(e) = apns.send(&token.push_token, &notification).await {
                    tracing::warn!(token = %token.push_token, error = %e, "APNs push failed");
                }
            }
        });
    }
}
```

## Server Init

`server/src/main.rs` — after config parsing:

```rust
let apns_client = apns::ApnsClient::new(&config).map(Arc::new);
if apns_client.is_some() {
    tracing::info!("APNs push notifications enabled");
} else {
    tracing::info!("APNs not configured — polling-only mode");
}
```

## New Queries

`server/src/db/queries.rs`:
- `list_push_tokens(conn, platform: Option<&str>) -> Vec<PushToken>`
- `delete_push_token(conn, token: &str)`

## Files Modified

| File | Action |
|---|---|
| `server/Cargo.toml` | modify — add a2 (or reqwest+jsonwebtoken) |
| `server/src/config.rs` | modify — add optional APNs fields |
| `server/src/router.rs` | modify — add apns_client to AppState |
| `server/src/main.rs` | modify — init ApnsClient |
| `server/src/apns.rs` | **create** — APNs client module |
| `server/src/handlers/events.rs` | modify — spawn APNs dispatch after COMMIT |
| `server/src/db/queries.rs` | modify — list_push_tokens + delete_push_token |
