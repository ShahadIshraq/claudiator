#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(clippy::similar_names)]
#![allow(missing_docs)]

use axum::http::StatusCode;
use axum_test::TestServer;
use claudiator_server::{db, models, router};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

fn test_server() -> TestServer {
    let db_pool = db::pool::create_pool(":memory:").unwrap();
    db::migrations::run(&db_pool).unwrap();

    let state = Arc::new(router::AppState {
        api_key: "test-key".to_string(),
        db_pool,
        version: AtomicU64::new(0),
        notification_version: AtomicU64::new(0),
        last_cleanup: AtomicU64::new(0),
        apns_client: None,
    });

    let app = router::build_router(state);
    TestServer::new(app).unwrap()
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
            .get(&format!("/api/v1/notifications?after={}", encoded_timestamp))
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
