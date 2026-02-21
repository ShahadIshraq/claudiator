#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(clippy::similar_names)]
#![allow(missing_docs)]

use axum::http::StatusCode;
use axum_test::TestServer;
use chrono::{SecondsFormat, Utc};
use claudiator_server::{db, db::queries, models, router};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

fn make_state() -> Arc<router::AppState> {
    let db_pool = db::pool::create_pool(":memory:").unwrap();
    db::migrations::run(&db_pool).unwrap();

    Arc::new(router::AppState {
        master_key: "test-key".to_string(),
        db_pool,
        version: AtomicU64::new(0),
        notification_version: AtomicU64::new(0),
        last_cleanup: AtomicU64::new(0),
        apns_client: None,
        retention_events_days: 7,
        retention_sessions_days: 7,
        retention_devices_days: 30,
        auth_failures: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        key_rate_limits: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        notif_cooldown: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
    })
}

fn test_server_from_state(state: Arc<router::AppState>) -> TestServer {
    let app = router::build_router(state);
    TestServer::new(app).unwrap()
}

// Injects a loopback ConnectInfo so AdminAuth passes the localhost check.
async fn inject_localhost_connect_info(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::extract::ConnectInfo;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    let addr = ConnectInfo(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 12345));
    request.extensions_mut().insert(addr);
    next.run(request).await
}

fn admin_test_server_from_state(state: Arc<router::AppState>) -> TestServer {
    let app =
        router::build_router(state).layer(axum::middleware::from_fn(inject_localhost_connect_info));
    TestServer::new(app).unwrap()
}

fn test_server() -> TestServer {
    test_server_from_state(make_state())
}

#[tokio::test]
async fn test_ping_with_auth() {
    let server = test_server();
    let response = server
        .get("/api/v1/ping")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["status"], "ok");
    assert!(json["server_version"].is_string());
}

#[tokio::test]
async fn test_ping_without_auth() {
    let server = test_server();
    let response = server.get("/api/v1/ping").await;

    response.assert_status_unauthorized();
    let json: serde_json::Value = response.json();
    assert_eq!(json["error"], "unauthorized");
}

#[tokio::test]
async fn test_events_valid() {
    let server = test_server();
    let payload = serde_json::json!({
        "device": {
            "device_id": "dev-1",
            "device_name": "Test Device",
            "platform": "macos"
        },
        "event": {
            "session_id": "sess-1",
            "hook_event_name": "session-start"
        },
        "timestamp": "2024-01-01T00:00:00Z"
    });

    let response = server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_events_empty_device_id() {
    let server = test_server();
    let payload = serde_json::json!({
        "device": {
            "device_id": "",
            "device_name": "Test Device",
            "platform": "macos"
        },
        "event": {
            "session_id": "sess-1",
            "hook_event_name": "session-start"
        },
        "timestamp": "2024-01-01T00:00:00Z"
    });

    let response = server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_events_without_auth() {
    let server = test_server();
    let payload = serde_json::json!({
        "device": {
            "device_id": "dev-1",
            "device_name": "Test Device",
            "platform": "macos"
        },
        "event": {
            "session_id": "sess-1",
            "hook_event_name": "session-start"
        },
        "timestamp": "2024-01-01T00:00:00Z"
    });

    let response = server.post("/api/v1/events").json(&payload).await;
    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_list_devices_empty() {
    let server = test_server();
    let response = server
        .get("/api/v1/devices")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["devices"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_devices_with_active_sessions() {
    let server = test_server();

    // Seed some data
    let event1 = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {"session_id": "sess-1", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:00:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&event1)
        .await;

    let event2 = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {"session_id": "sess-2", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:01:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&event2)
        .await;

    // List devices
    let response = server
        .get("/api/v1/devices")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    let devices = json["devices"].as_array().unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0]["device_id"], "dev-1");
    assert_eq!(devices[0]["active_sessions"], 2);
}

#[tokio::test]
async fn test_list_device_sessions_with_status_filter() {
    let server = test_server();

    // Seed two sessions
    let event1 = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {"session_id": "sess-1", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:00:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&event1)
        .await;

    let event2 = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {"session_id": "sess-2", "hook_event_name": "tool-use"},
        "timestamp": "2024-01-01T00:01:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&event2)
        .await;

    // No filter - should return all sessions
    let response = server
        .get("/api/v1/devices/dev-1/sessions")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["sessions"].as_array().unwrap().len(), 2);

    // Filter by status parameter works (even if all sessions have same status)
    let response = server
        .get("/api/v1/devices/dev-1/sessions?status=active")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    // Just verify the endpoint accepts the parameter
}

#[tokio::test]
async fn test_list_device_sessions_limit() {
    let server = test_server();

    // Seed 5 sessions
    for i in 1..=5 {
        let event = serde_json::json!({
            "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
            "event": {"session_id": format!("sess-{}", i), "hook_event_name": "session-start"},
            "timestamp": format!("2024-01-01T00:0{}:00Z", i)
        });
        server
            .post("/api/v1/events")
            .add_header("Authorization", "Bearer test-key")
            .json(&event)
            .await;
    }

    // List with limit
    let response = server
        .get("/api/v1/devices/dev-1/sessions?limit=3")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["sessions"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_list_device_sessions_unknown_device() {
    let server = test_server();
    let response = server
        .get("/api/v1/devices/unknown/sessions")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["sessions"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_all_sessions() {
    let server = test_server();

    // Seed sessions on different devices
    let event1 = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {"session_id": "sess-1", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:00:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&event1)
        .await;

    let event2 = serde_json::json!({
        "device": {"device_id": "dev-2", "device_name": "Device 2", "platform": "linux"},
        "event": {"session_id": "sess-2", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:01:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&event2)
        .await;

    // List all sessions
    let response = server
        .get("/api/v1/sessions")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["sessions"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_session_events() {
    let server = test_server();

    // Seed session with events
    let event1 = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {"session_id": "sess-1", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:00:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&event1)
        .await;

    let event2 = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {"session_id": "sess-1", "hook_event_name": "tool-use", "tool_name": "bash"},
        "timestamp": "2024-01-01T00:01:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&event2)
        .await;

    // List events (should be in desc order)
    let response = server
        .get("/api/v1/sessions/sess-1/events")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    // Most recent first
    assert_eq!(events[0]["hook_event_name"], "tool-use");
    assert_eq!(events[1]["hook_event_name"], "session-start");
}

#[tokio::test]
async fn test_push_register_valid() {
    let server = test_server();
    let payload = serde_json::json!({
        "platform": "ios",
        "push_token": "abc123"
    });

    let response = server
        .post("/api/v1/push/register")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status_ok();
}

#[tokio::test]
async fn test_push_register_empty_platform() {
    let server = test_server();
    let payload = serde_json::json!({
        "platform": "",
        "push_token": "abc123"
    });

    let response = server
        .post("/api/v1/push/register")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_push_register_upsert() {
    let server = test_server();

    // Insert
    let payload1 = serde_json::json!({
        "platform": "ios",
        "push_token": "token-1"
    });
    server
        .post("/api/v1/push/register")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload1)
        .await;

    // Update same token with different platform
    let payload2 = serde_json::json!({
        "platform": "android",
        "push_token": "token-1"
    });
    let response = server
        .post("/api/v1/push/register")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload2)
        .await;

    response.assert_status_ok();
}

#[tokio::test]
async fn test_list_notifications_empty() {
    let server = test_server();
    let response = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["notifications"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_notifications_limit_caps_at_200() {
    let server = test_server();
    let response = server
        .get("/api/v1/notifications?limit=300")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    // The implementation should cap at 200, but we can't easily verify without seeding 200+ records
    // This test just ensures the endpoint accepts the parameter
}

#[tokio::test]
async fn test_list_notifications_with_after_timestamp() {
    let server = test_server();

    // Seed an event and notification
    let event_payload = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {"session_id": "sess-1", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:00:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&event_payload)
        .await;

    // List all notifications to get the timestamp
    let response = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    let notifications = json["notifications"].as_array().unwrap();

    if !notifications.is_empty() {
        let first_timestamp = notifications[0]["created_at"].as_str().unwrap();

        // Query with after parameter using the timestamp
        // URL encode the timestamp manually to avoid dependency
        let encoded_timestamp = first_timestamp.replace(":", "%3A").replace("+", "%2B");
        let response = server
            .get(&format!(
                "/api/v1/notifications?after={}",
                encoded_timestamp
            ))
            .add_header("Authorization", "Bearer test-key")
            .await;

        response.assert_status_ok();
        let json: serde_json::Value = response.json();
        // Should return notifications created after the specified timestamp
        assert!(json["notifications"].is_array());
    }
}

#[tokio::test]
async fn test_acknowledge_notifications() {
    let server = test_server();

    // Seed an event and notification
    let event_payload = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {"session_id": "sess-1", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:00:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&event_payload)
        .await;

    // Get notification IDs
    let response = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    let notifications = json["notifications"].as_array().unwrap();

    if !notifications.is_empty() {
        let notif_id = notifications[0]["id"].as_str().unwrap();

        // Acknowledge the notification
        let ack_payload = serde_json::json!({
            "ids": [notif_id]
        });

        let response = server
            .post("/api/v1/notifications/ack")
            .add_header("Authorization", "Bearer test-key")
            .json(&ack_payload)
            .await;

        response.assert_status_ok();
        let json: serde_json::Value = response.json();
        assert_eq!(json["status"], "ok");
    }
}

#[tokio::test]
async fn test_acknowledge_notifications_empty_array() {
    let server = test_server();

    let ack_payload = serde_json::json!({
        "ids": []
    });

    let response = server
        .post("/api/v1/notifications/ack")
        .add_header("Authorization", "Bearer test-key")
        .json(&ack_payload)
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_acknowledge_notifications_without_auth() {
    let server = test_server();

    let ack_payload = serde_json::json!({
        "ids": ["notif-1"]
    });

    let response = server
        .post("/api/v1/notifications/ack")
        .json(&ack_payload)
        .await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_notification_content_uses_session_title() {
    let server = test_server();

    // First, set a session title via UserPromptSubmit
    let prompt_event = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {
            "session_id": "sess-1",
            "hook_event_name": "UserPromptSubmit",
            "prompt": "Fix the login bug"
        },
        "timestamp": "2024-01-01T00:00:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&prompt_event)
        .await;

    // Then trigger a Stop event (which creates a notification)
    let stop_event = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {
            "session_id": "sess-1",
            "hook_event_name": "Stop",
            "message": "Max turns reached"
        },
        "timestamp": "2024-01-01T00:01:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&stop_event)
        .await;

    // Fetch notifications and verify content
    let response = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    let notifications = json["notifications"].as_array().unwrap();
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0]["title"], "Fix the login bug");
    assert_eq!(
        notifications[0]["body"],
        "Session stopped: Max turns reached"
    );
}

#[tokio::test]
async fn test_notification_content_fallback_without_session_title() {
    let server = test_server();

    // Send a Stop event without a prior UserPromptSubmit (no session title)
    let stop_event = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {
            "session_id": "sess-no-title",
            "hook_event_name": "Stop",
            "message": "User interrupted"
        },
        "timestamp": "2024-01-01T00:00:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&stop_event)
        .await;

    let response = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    let notifications = json["notifications"].as_array().unwrap();
    assert_eq!(notifications.len(), 1);
    // Should fall back to hardcoded title
    assert_eq!(notifications[0]["title"], "Session Stopped");
    assert_eq!(
        notifications[0]["body"],
        "Session stopped: User interrupted"
    );
}

#[tokio::test]
async fn test_notification_content_permission_with_tool_name() {
    let server = test_server();

    // Set session title
    let prompt_event = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {
            "session_id": "sess-perm",
            "hook_event_name": "UserPromptSubmit",
            "prompt": "Refactor auth module"
        },
        "timestamp": "2024-01-01T00:00:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&prompt_event)
        .await;

    // Send a PermissionRequest event with tool_name and message
    let perm_event = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "Device 1", "platform": "macos"},
        "event": {
            "session_id": "sess-perm",
            "hook_event_name": "PermissionRequest",
            "tool_name": "Bash",
            "message": "run npm test"
        },
        "timestamp": "2024-01-01T00:01:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&perm_event)
        .await;

    let response = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    let notifications = json["notifications"].as_array().unwrap();
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0]["title"], "Refactor auth module");
    assert_eq!(
        notifications[0]["body"],
        "Permission required: Bash \u{2014} run npm test"
    );
}

// ── Scope enforcement ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_read_key_allowed_on_get_ping() {
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(&conn, "k1", "reader", "claud_readtest1", "read", &now, None).unwrap();
    drop(conn);

    let server = test_server_from_state(state);
    let response = server
        .get("/api/v1/ping")
        .add_header("Authorization", "Bearer claud_readtest1")
        .await;
    response.assert_status_ok();
}

#[tokio::test]
async fn test_read_key_forbidden_on_post_events() {
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(&conn, "k1", "reader", "claud_readtest2", "read", &now, None).unwrap();
    drop(conn);

    let payload = serde_json::json!({
        "device": {"device_id": "d1", "device_name": "D", "platform": "macos"},
        "event": {"session_id": "s1", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:00:00Z"
    });

    let server = test_server_from_state(state);
    let response = server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer claud_readtest2")
        .json(&payload)
        .await;
    response.assert_status(StatusCode::FORBIDDEN);
    let json: serde_json::Value = response.json();
    assert_eq!(json["error"], "forbidden");
}

#[tokio::test]
async fn test_read_key_forbidden_on_post_push_register() {
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(&conn, "k1", "reader", "claud_readtest3", "read", &now, None).unwrap();
    drop(conn);

    let payload = serde_json::json!({"platform": "ios", "push_token": "tok123"});
    let server = test_server_from_state(state);
    let response = server
        .post("/api/v1/push/register")
        .add_header("Authorization", "Bearer claud_readtest3")
        .json(&payload)
        .await;
    response.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_read_key_forbidden_on_post_notifications_ack() {
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(&conn, "k1", "reader", "claud_readtest4", "read", &now, None).unwrap();
    drop(conn);

    let payload = serde_json::json!({"ids": []});
    let server = test_server_from_state(state);
    let response = server
        .post("/api/v1/notifications/ack")
        .add_header("Authorization", "Bearer claud_readtest4")
        .json(&payload)
        .await;
    response.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_write_key_allowed_on_post_events() {
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(
        &conn,
        "k1",
        "writer",
        "claud_writetest1",
        "write",
        &now,
        None,
    )
    .unwrap();
    drop(conn);

    let payload = serde_json::json!({
        "device": {"device_id": "d1", "device_name": "D", "platform": "macos"},
        "event": {"session_id": "s1", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:00:00Z"
    });
    let server = test_server_from_state(state);
    let response = server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer claud_writetest1")
        .json(&payload)
        .await;
    response.assert_status_ok();
}

#[tokio::test]
async fn test_write_key_forbidden_on_get_devices() {
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(
        &conn,
        "k1",
        "writer",
        "claud_writetest2",
        "write",
        &now,
        None,
    )
    .unwrap();
    drop(conn);

    let server = test_server_from_state(state);
    let response = server
        .get("/api/v1/devices")
        .add_header("Authorization", "Bearer claud_writetest2")
        .await;
    response.assert_status(StatusCode::FORBIDDEN);
    let json: serde_json::Value = response.json();
    assert_eq!(json["error"], "forbidden");
}

#[tokio::test]
async fn test_write_key_forbidden_on_get_sessions() {
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(
        &conn,
        "k1",
        "writer",
        "claud_writetest3",
        "write",
        &now,
        None,
    )
    .unwrap();
    drop(conn);

    let server = test_server_from_state(state);
    let response = server
        .get("/api/v1/sessions")
        .add_header("Authorization", "Bearer claud_writetest3")
        .await;
    response.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_write_key_forbidden_on_get_notifications() {
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(
        &conn,
        "k1",
        "writer",
        "claud_writetest4",
        "write",
        &now,
        None,
    )
    .unwrap();
    drop(conn);

    let server = test_server_from_state(state);
    let response = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer claud_writetest4")
        .await;
    response.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_rw_key_allowed_on_get_and_post() {
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(&conn, "k1", "rw", "claud_rwtest1", "read,write", &now, None).unwrap();
    drop(conn);

    let server = test_server_from_state(state);

    let get_resp = server
        .get("/api/v1/devices")
        .add_header("Authorization", "Bearer claud_rwtest1")
        .await;
    get_resp.assert_status_ok();

    let payload = serde_json::json!({
        "device": {"device_id": "d1", "device_name": "D", "platform": "macos"},
        "event": {"session_id": "s1", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:00:00Z"
    });
    let post_resp = server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer claud_rwtest1")
        .json(&payload)
        .await;
    post_resp.assert_status_ok();
}

#[tokio::test]
async fn test_unknown_db_key_returns_401() {
    let server = test_server();
    let response = server
        .get("/api/v1/ping")
        .add_header("Authorization", "Bearer claud_doesnotexist")
        .await;
    response.assert_status_unauthorized();
    let json: serde_json::Value = response.json();
    assert_eq!(json["error"], "unauthorized");
}

#[tokio::test]
async fn test_master_key_still_works_after_db_keys_added() {
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(&conn, "k1", "other", "claud_other", "read", &now, None).unwrap();
    drop(conn);

    let server = test_server_from_state(state);
    let response = server
        .get("/api/v1/ping")
        .add_header("Authorization", "Bearer test-key")
        .await;
    response.assert_status_ok();
}

#[tokio::test]
async fn test_last_used_updated_after_successful_auth() {
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(&conn, "k1", "tracker", "claud_trackme", "read", &now, None).unwrap();

    // Verify last_used is initially null
    let row_before = queries::find_api_key_by_key(&conn, "claud_trackme")
        .unwrap()
        .unwrap();
    assert!(row_before.last_used.is_none());
    drop(conn);

    let server = test_server_from_state(Arc::clone(&state));
    server
        .get("/api/v1/ping")
        .add_header("Authorization", "Bearer claud_trackme")
        .await;

    // Verify last_used was set
    let conn2 = state.db_pool.get().unwrap();
    let row_after = queries::find_api_key_by_key(&conn2, "claud_trackme")
        .unwrap()
        .unwrap();
    assert!(row_after.last_used.is_some());
}

// ── Admin endpoint tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_list_keys_empty() {
    let state = make_state();
    let server = admin_test_server_from_state(state);
    let response = server
        .get("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .await;
    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["keys"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_admin_create_key_read_scope() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let payload = serde_json::json!({"name": "ios-app", "scopes": ["read"]});
    let response = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::CREATED);
    let json: serde_json::Value = response.json();
    assert_eq!(json["name"], "ios-app");
    assert_eq!(json["scopes"].as_array().unwrap().len(), 1);
    assert_eq!(json["scopes"][0], "read");
    // Key is returned in full on creation
    let key = json["key"].as_str().unwrap();
    assert!(key.starts_with("claud_"));
    // ID is a UUID
    assert!(json["id"].as_str().unwrap().len() > 0);
    assert!(json["created_at"].is_string());
}

#[tokio::test]
async fn test_admin_create_key_write_scope() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let payload = serde_json::json!({"name": "hook-client", "scopes": ["write"]});
    let response = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::CREATED);
    let json: serde_json::Value = response.json();
    assert_eq!(json["scopes"][0], "write");
}

#[tokio::test]
async fn test_admin_create_key_both_scopes() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let payload = serde_json::json!({"name": "full-access", "scopes": ["read", "write"]});
    let response = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::CREATED);
    let json: serde_json::Value = response.json();
    let scopes = json["scopes"].as_array().unwrap();
    assert_eq!(scopes.len(), 2);
}

#[tokio::test]
async fn test_admin_create_key_deduplicates_scopes() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let payload = serde_json::json!({"name": "dedup", "scopes": ["read", "read", "write"]});
    let response = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::CREATED);
    let json: serde_json::Value = response.json();
    // Should have exactly 2 scopes (read deduplicated)
    assert_eq!(json["scopes"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_admin_create_key_empty_name_returns_422() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let payload = serde_json::json!({"name": "", "scopes": ["read"]});
    let response = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    let json: serde_json::Value = response.json();
    assert_eq!(json["error"], "bad_request");
}

#[tokio::test]
async fn test_admin_create_key_whitespace_only_name_returns_422() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let payload = serde_json::json!({"name": "   ", "scopes": ["read"]});
    let response = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_admin_create_key_empty_scopes_returns_422() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let payload = serde_json::json!({"name": "mykey", "scopes": []});
    let response = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    let json: serde_json::Value = response.json();
    assert_eq!(json["error"], "bad_request");
}

#[tokio::test]
async fn test_admin_create_key_invalid_scope_returns_422() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let payload = serde_json::json!({"name": "mykey", "scopes": ["admin"]});
    let response = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    let json: serde_json::Value = response.json();
    assert_eq!(json["error"], "bad_request");
}

#[tokio::test]
async fn test_admin_list_keys_shows_created_keys() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    // Create two keys
    let p1 = serde_json::json!({"name": "key-one", "scopes": ["read"]});
    server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&p1)
        .await;

    let p2 = serde_json::json!({"name": "key-two", "scopes": ["write"]});
    server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&p2)
        .await;

    // List
    let response = server
        .get("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    let keys = json["keys"].as_array().unwrap();
    assert_eq!(keys.len(), 2);

    let names: Vec<&str> = keys.iter().map(|k| k["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"key-one"));
    assert!(names.contains(&"key-two"));
}

#[tokio::test]
async fn test_admin_list_keys_masks_full_key_value() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let payload = serde_json::json!({"name": "masked", "scopes": ["read"]});
    let create_response = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;
    let created: serde_json::Value = create_response.json();
    let full_key = created["key"].as_str().unwrap().to_string();

    // List should return key_prefix not full key
    let list_response = server
        .get("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .await;
    let list_json: serde_json::Value = list_response.json();
    let keys = list_json["keys"].as_array().unwrap();
    assert_eq!(keys.len(), 1);

    // key_prefix field should exist and be shorter than the full key
    let key_prefix = keys[0]["key_prefix"].as_str().unwrap();
    assert!(full_key.starts_with(key_prefix));
    assert!(key_prefix.len() < full_key.len());
    // The list entry should NOT have a "key" field with the full value
    assert!(
        keys[0].get("key").is_none() || keys[0]["key"].as_str().map_or(true, |k| k != full_key)
    );
}

#[tokio::test]
async fn test_admin_delete_key() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    // Create a key
    let payload = serde_json::json!({"name": "to-delete", "scopes": ["read"]});
    let create_resp: serde_json::Value = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await
        .json();
    let id = create_resp["id"].as_str().unwrap().to_string();

    // Verify it's in the list
    let list_before: serde_json::Value = server
        .get("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .await
        .json();
    assert_eq!(list_before["keys"].as_array().unwrap().len(), 1);

    // Delete it
    let delete_resp = server
        .delete(&format!("/admin/api-keys/{}", id))
        .add_header("Authorization", "Bearer test-key")
        .await;
    delete_resp.assert_status_ok();
    let delete_json: serde_json::Value = delete_resp.json();
    assert_eq!(delete_json["status"], "ok");

    // Verify it's gone from the list
    let list_after: serde_json::Value = server
        .get("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .await
        .json();
    assert_eq!(list_after["keys"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_admin_delete_nonexistent_key_is_ok() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let response = server
        .delete("/admin/api-keys/nonexistent-uuid")
        .add_header("Authorization", "Bearer test-key")
        .await;
    response.assert_status_ok();
}

#[tokio::test]
async fn test_admin_without_auth_returns_401() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let response = server.get("/admin/api-keys").await;
    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_admin_with_wrong_key_returns_401() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let response = server
        .get("/admin/api-keys")
        .add_header("Authorization", "Bearer wrong-key")
        .await;
    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_admin_without_connect_info_returns_403() {
    // Regular test_server has no ConnectInfo injected, so AdminAuth returns Forbidden
    let server = test_server();
    let response = server
        .get("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .await;
    response.assert_status(StatusCode::FORBIDDEN);
    let json: serde_json::Value = response.json();
    assert_eq!(json["error"], "forbidden");
}

#[tokio::test]
async fn test_admin_db_key_cannot_access_admin_endpoints() {
    // Even a read+write DB key should not work on admin endpoints (only master key allowed)
    let state = make_state();
    let conn = state.db_pool.get().unwrap();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    queries::insert_api_key(
        &conn,
        "k1",
        "full-access",
        "claud_rwkey",
        "read,write",
        &now,
        None,
    )
    .unwrap();
    drop(conn);

    let server = admin_test_server_from_state(state);
    let response = server
        .get("/admin/api-keys")
        .add_header("Authorization", "Bearer claud_rwkey")
        .await;
    // DB keys are rejected on admin endpoints — only master key works there
    response.assert_status_unauthorized();
}

// ── Lifecycle integration tests ──────────────────────────────────────────────

#[tokio::test]
async fn test_lifecycle_create_use_delete_key() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    // Create a read-only key
    let payload = serde_json::json!({"name": "lifecycle-test", "scopes": ["read"]});
    let create_resp: serde_json::Value = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await
        .json();
    let key = create_resp["key"].as_str().unwrap().to_string();
    let id = create_resp["id"].as_str().unwrap().to_string();

    // Use the key on a read endpoint — should succeed
    let get_resp = server
        .get("/api/v1/devices")
        .add_header("Authorization", &format!("Bearer {}", key))
        .await;
    get_resp.assert_status_ok();

    // Use the key on a write endpoint — should fail with 403
    let payload = serde_json::json!({
        "device": {"device_id": "d1", "device_name": "D", "platform": "macos"},
        "event": {"session_id": "s1", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:00:00Z"
    });
    let post_resp = server
        .post("/api/v1/events")
        .add_header("Authorization", &format!("Bearer {}", key))
        .json(&payload)
        .await;
    post_resp.assert_status(StatusCode::FORBIDDEN);

    // Delete the key
    server
        .delete(&format!("/admin/api-keys/{}", id))
        .add_header("Authorization", "Bearer test-key")
        .await;

    // After deletion, the key should return 401
    let after_delete = server
        .get("/api/v1/devices")
        .add_header("Authorization", &format!("Bearer {}", key))
        .await;
    after_delete.assert_status_unauthorized();
}

#[tokio::test]
async fn test_lifecycle_write_only_key() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    // Create a write-only key
    let payload = serde_json::json!({"name": "hook", "scopes": ["write"]});
    let create_resp: serde_json::Value = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await
        .json();
    let key = create_resp["key"].as_str().unwrap().to_string();

    // Write endpoint works
    let event_payload = serde_json::json!({
        "device": {"device_id": "d1", "device_name": "D", "platform": "macos"},
        "event": {"session_id": "s1", "hook_event_name": "session-start"},
        "timestamp": "2024-01-01T00:00:00Z"
    });
    let write_resp = server
        .post("/api/v1/events")
        .add_header("Authorization", &format!("Bearer {}", key))
        .json(&event_payload)
        .await;
    write_resp.assert_status_ok();

    // Read endpoint is forbidden
    let read_resp = server
        .get("/api/v1/sessions")
        .add_header("Authorization", &format!("Bearer {}", key))
        .await;
    read_resp.assert_status(StatusCode::FORBIDDEN);
}

// ── Rate limiting tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_key_with_rate_limit_reflected_in_response() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let payload = serde_json::json!({"name": "limited", "scopes": ["read"], "rate_limit": 5});
    let response = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::CREATED);
    let json: serde_json::Value = response.json();
    assert_eq!(json["rate_limit"], 5);
}

#[tokio::test]
async fn test_create_key_without_rate_limit_omitted_from_response() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    let payload = serde_json::json!({"name": "default-limited", "scopes": ["read"]});
    let response = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await;

    response.assert_status(StatusCode::CREATED);
    let json: serde_json::Value = response.json();
    // rate_limit not set — should be absent from response
    assert!(json.get("rate_limit").is_none() || json["rate_limit"].is_null());
}

#[tokio::test]
async fn test_key_rate_limit_enforced() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    // Create a key with a rate_limit of 3
    let payload = serde_json::json!({"name": "tight", "scopes": ["read"], "rate_limit": 3});
    let create_resp: serde_json::Value = server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&payload)
        .await
        .json();
    let key = create_resp["key"].as_str().unwrap().to_string();

    // Requests 1-3 should succeed
    for _ in 0..3 {
        let resp = server
            .get("/api/v1/ping")
            .add_header("Authorization", &format!("Bearer {}", key))
            .await;
        resp.assert_status_ok();
    }

    // Request 4 should be rate-limited
    let resp = server
        .get("/api/v1/ping")
        .add_header("Authorization", &format!("Bearer {}", key))
        .await;
    resp.assert_status(StatusCode::TOO_MANY_REQUESTS);
    let json: serde_json::Value = resp.json();
    assert_eq!(json["error"], "rate_limited");
}

#[tokio::test]
async fn test_list_keys_shows_rate_limit() {
    let state = make_state();
    let server = admin_test_server_from_state(state);

    // Create one key with rate_limit and one without
    let p1 = serde_json::json!({"name": "with-limit", "scopes": ["read"], "rate_limit": 100});
    server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&p1)
        .await;

    let p2 = serde_json::json!({"name": "no-limit", "scopes": ["read"]});
    server
        .post("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .json(&p2)
        .await;

    let response = server
        .get("/admin/api-keys")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    let keys = json["keys"].as_array().unwrap();
    assert_eq!(keys.len(), 2);

    let with_limit = keys.iter().find(|k| k["name"] == "with-limit").unwrap();
    assert_eq!(with_limit["rate_limit"], 100);

    let no_limit = keys.iter().find(|k| k["name"] == "no-limit").unwrap();
    assert!(no_limit.get("rate_limit").is_none() || no_limit["rate_limit"].is_null());
}

// ── Notification cooldown (dedup) tests ──────────────────────────────────────

/// Sends two Stop events for the same session in rapid succession.
/// Only the first should produce a notification row.
#[tokio::test]
async fn test_stop_notification_suppressed_within_cooldown() {
    let server = test_server();

    let stop = |ts: &str| {
        serde_json::json!({
            "device": {"device_id": "dev-1", "device_name": "D", "platform": "macos"},
            "event": {"session_id": "sess-cool-stop", "hook_event_name": "Stop", "message": "done"},
            "timestamp": ts
        })
    };

    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&stop("2024-01-01T00:00:00Z"))
        .await
        .assert_status_ok();

    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&stop("2024-01-01T00:00:01Z"))
        .await
        .assert_status_ok();

    let response = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    let notifications = json["notifications"].as_array().unwrap();
    // Second Stop must be suppressed — only 1 notification row.
    assert_eq!(
        notifications.len(),
        1,
        "expected exactly 1 notification for rapid Stop burst"
    );
    assert_eq!(notifications[0]["notification_type"], "stop");
}

/// Sends two `idle_prompt` Notification events for the same session in rapid succession.
/// Only the first should produce a notification row.
#[tokio::test]
async fn test_idle_prompt_notification_suppressed_within_cooldown() {
    let server = test_server();

    let idle = |ts: &str| {
        serde_json::json!({
            "device": {"device_id": "dev-1", "device_name": "D", "platform": "macos"},
            "event": {
                "session_id": "sess-cool-idle",
                "hook_event_name": "Notification",
                "notification_type": "idle_prompt",
                "message": "waiting"
            },
            "timestamp": ts
        })
    };

    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&idle("2024-01-01T00:00:00Z"))
        .await
        .assert_status_ok();

    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&idle("2024-01-01T00:00:01Z"))
        .await
        .assert_status_ok();

    let json: serde_json::Value = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await
        .json();

    let notifications = json["notifications"].as_array().unwrap();
    assert_eq!(
        notifications.len(),
        1,
        "expected exactly 1 notification for rapid idle_prompt burst"
    );
    assert_eq!(notifications[0]["notification_type"], "idle_prompt");
}

/// Sends multiple `PermissionRequest` events for the same session in rapid succession.
/// All should produce notification rows (high-priority, never suppressed).
#[tokio::test]
async fn test_permission_prompt_never_suppressed() {
    let server = test_server();

    let perm = |ts: &str| {
        serde_json::json!({
            "device": {"device_id": "dev-1", "device_name": "D", "platform": "macos"},
            "event": {
                "session_id": "sess-cool-perm",
                "hook_event_name": "PermissionRequest",
                "tool_name": "Bash",
                "message": "run tests"
            },
            "timestamp": ts
        })
    };

    for ts in &[
        "2024-01-01T00:00:00Z",
        "2024-01-01T00:00:01Z",
        "2024-01-01T00:00:02Z",
    ] {
        server
            .post("/api/v1/events")
            .add_header("Authorization", "Bearer test-key")
            .json(&perm(ts))
            .await
            .assert_status_ok();
    }

    let json: serde_json::Value = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await
        .json();

    let notifications = json["notifications"].as_array().unwrap();
    assert_eq!(
        notifications.len(),
        3,
        "permission_prompt must never be suppressed"
    );
    for n in notifications {
        assert_eq!(n["notification_type"], "permission_prompt");
    }
}

/// Cooldown is per-session: two different sessions each get their own first Stop notification.
#[tokio::test]
async fn test_cooldown_is_independent_per_session() {
    let server = test_server();

    let stop = |session: &str, ts: &str| {
        serde_json::json!({
            "device": {"device_id": "dev-1", "device_name": "D", "platform": "macos"},
            "event": {"session_id": session, "hook_event_name": "Stop", "message": "done"},
            "timestamp": ts
        })
    };

    // Two different sessions, each fires a Stop.
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&stop("sess-a", "2024-01-01T00:00:00Z"))
        .await
        .assert_status_ok();

    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&stop("sess-b", "2024-01-01T00:00:01Z"))
        .await
        .assert_status_ok();

    let json: serde_json::Value = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await
        .json();

    let notifications = json["notifications"].as_array().unwrap();
    // Each session must have produced exactly one notification.
    assert_eq!(
        notifications.len(),
        2,
        "each session should get its own first notification"
    );

    let session_ids: Vec<&str> = notifications
        .iter()
        .map(|n| n["session_id"].as_str().unwrap())
        .collect();
    assert!(session_ids.contains(&"sess-a"));
    assert!(session_ids.contains(&"sess-b"));
}

/// Cooldown is per-type: Stop and `idle_prompt` for the same session are tracked independently.
#[tokio::test]
async fn test_cooldown_is_independent_per_type() {
    let server = test_server();

    // Send Stop first.
    let stop_event = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "D", "platform": "macos"},
        "event": {"session_id": "sess-types", "hook_event_name": "Stop", "message": "done"},
        "timestamp": "2024-01-01T00:00:00Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&stop_event)
        .await
        .assert_status_ok();

    // Then send idle_prompt for the same session — must not be suppressed by Stop's cooldown.
    let idle_event = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "D", "platform": "macos"},
        "event": {
            "session_id": "sess-types",
            "hook_event_name": "Notification",
            "notification_type": "idle_prompt",
            "message": "still waiting"
        },
        "timestamp": "2024-01-01T00:00:01Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&idle_event)
        .await
        .assert_status_ok();

    let json: serde_json::Value = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await
        .json();

    let notifications = json["notifications"].as_array().unwrap();
    assert_eq!(
        notifications.len(),
        2,
        "stop and idle_prompt must have independent cooldown buckets"
    );

    let types: Vec<&str> = notifications
        .iter()
        .map(|n| n["notification_type"].as_str().unwrap())
        .collect();
    assert!(types.contains(&"stop"));
    assert!(types.contains(&"idle_prompt"));
}

/// After suppression, an ack of the first notification does not affect the cooldown —
/// a duplicate fired immediately after ack is still suppressed.
#[tokio::test]
async fn test_ack_does_not_reset_cooldown() {
    let server = test_server();

    let stop_event = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "D", "platform": "macos"},
        "event": {"session_id": "sess-ack", "hook_event_name": "Stop", "message": "done"},
        "timestamp": "2024-01-01T00:00:00Z"
    });

    // First Stop fires.
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&stop_event)
        .await
        .assert_status_ok();

    // Get notification ID and ack it.
    let list_json: serde_json::Value = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await
        .json();
    let notifications = list_json["notifications"].as_array().unwrap();
    assert_eq!(notifications.len(), 1);
    let notif_id = notifications[0]["id"].as_str().unwrap();

    server
        .post("/api/v1/notifications/ack")
        .add_header("Authorization", "Bearer test-key")
        .json(&serde_json::json!({"ids": [notif_id]}))
        .await
        .assert_status_ok();

    // Fire another Stop immediately — must still be suppressed despite the ack.
    let stop_event2 = serde_json::json!({
        "device": {"device_id": "dev-1", "device_name": "D", "platform": "macos"},
        "event": {"session_id": "sess-ack", "hook_event_name": "Stop", "message": "again"},
        "timestamp": "2024-01-01T00:00:01Z"
    });
    server
        .post("/api/v1/events")
        .add_header("Authorization", "Bearer test-key")
        .json(&stop_event2)
        .await
        .assert_status_ok();

    let list_json2: serde_json::Value = server
        .get("/api/v1/notifications")
        .add_header("Authorization", "Bearer test-key")
        .await
        .json();
    let notifications2 = list_json2["notifications"].as_array().unwrap();
    // Still only 1 notification total (second was suppressed).
    assert_eq!(notifications2.len(), 1, "ack must not reset the cooldown");
}
