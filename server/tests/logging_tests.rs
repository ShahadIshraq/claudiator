#![allow(clippy::unwrap_used)]
#![allow(missing_docs)]

use axum_test::TestServer;
use claudiator_server::{db, router};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

fn test_server() -> TestServer {
    let db_pool = db::pool::create_pool(":memory:").unwrap();
    db::migrations::run(&db_pool).unwrap();

    let state = Arc::new(router::AppState {
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
    });

    let app = router::build_router(state);
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn trace_layer_does_not_break_ping() {
    let server = test_server();
    let response = server
        .get("/api/v1/ping")
        .add_header("Authorization", "Bearer test-key")
        .await;

    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn trace_layer_does_not_break_events() {
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
}
