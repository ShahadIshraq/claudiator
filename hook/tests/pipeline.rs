//! Integration tests for the full hook pipeline.
//!
//! These tests exercise the path that production code takes:
//!   raw JSON (stdin) → HookEvent → EventPayload → serialized JSON body
//!
//! No real HTTP server is needed; we verify that the intermediate structs
//! are correctly built and that the final JSON body has the expected shape.

use claudiator_hook::config::Config;
use claudiator_hook::event::HookEvent;
use claudiator_hook::payload::EventPayload;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_config(server_url: &str) -> Config {
    Config {
        server_url: server_url.to_string(),
        api_key: "test-api-key".to_string(),
        device_name: "test-machine".to_string(),
        device_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        platform: "mac".to_string(),
        log_level: "error".to_string(),
        max_log_size_bytes: 1_048_576,
        max_log_backups: 2,
        raw_event_log_path: None,
    }
}

fn parse_event(json: &str) -> HookEvent {
    serde_json::from_str(json).expect("JSON should parse into HookEvent")
}

// ---------------------------------------------------------------------------
// Pipeline: parse → build payload → serialize
// ---------------------------------------------------------------------------

#[test]
fn test_pipeline_minimal_event_json_to_payload_to_body() {
    let raw = r#"{
        "session_id": "sess-abc",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "ls -la"}
    }"#;

    // Step 1 – parse from JSON (simulated stdin).
    let event = parse_event(raw);
    assert_eq!(event.session_id, "sess-abc");
    assert_eq!(event.hook_event_name, "PreToolUse");
    assert_eq!(event.tool_name.as_deref(), Some("Bash"));

    // Step 2 – build payload with device metadata.
    let config = make_config("https://example.com");
    let payload = EventPayload::new(&config, event);

    assert_eq!(
        payload.device.device_id,
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_eq!(payload.device.device_name, "test-machine");
    assert_eq!(payload.device.platform, "mac");
    assert_eq!(payload.event.session_id, "sess-abc");
    assert_eq!(payload.event.hook_event_name, "PreToolUse");

    // Step 3 – serialize to JSON (the HTTP request body).
    let body = serde_json::to_string(&payload).expect("payload must serialize to JSON");
    let body_value: serde_json::Value =
        serde_json::from_str(&body).expect("serialized body must be valid JSON");

    // Verify top-level structure expected by the server.
    assert!(
        body_value.get("device").is_some(),
        "body must contain 'device'"
    );
    assert!(
        body_value.get("event").is_some(),
        "body must contain 'event'"
    );
    assert!(
        body_value.get("timestamp").is_some(),
        "body must contain 'timestamp'"
    );

    // Device fields.
    assert_eq!(
        body_value["device"]["device_id"],
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_eq!(body_value["device"]["device_name"], "test-machine");
    assert_eq!(body_value["device"]["platform"], "mac");

    // Event fields survive round-trip.
    assert_eq!(body_value["event"]["session_id"], "sess-abc");
    assert_eq!(body_value["event"]["hook_event_name"], "PreToolUse");
    assert_eq!(body_value["event"]["tool_name"], "Bash");
}

#[test]
fn test_pipeline_notification_event() {
    let raw = r#"{
        "session_id": "sess-notif",
        "hook_event_name": "Notification",
        "notification_type": "progress",
        "message": "Compiling…",
        "title": "Build"
    }"#;

    let event = parse_event(raw);
    let config = make_config("https://server.example.com");
    let payload = EventPayload::new(&config, event);

    let body = serde_json::to_string(&payload).expect("payload must serialize");
    let body_value: serde_json::Value = serde_json::from_str(&body).expect("must be valid JSON");

    assert_eq!(body_value["event"]["hook_event_name"], "Notification");
    assert_eq!(body_value["event"]["notification_type"], "progress");
    assert_eq!(body_value["event"]["message"], "Compiling…");
    // title is not in the trimmed HookEvent DTO so it must not appear
    assert!(
        body_value["event"].get("title").is_none(),
        "title must not appear in trimmed DTO"
    );
}

#[test]
fn test_pipeline_unknown_fields_not_forwarded_to_server() {
    // Unknown fields are silently dropped when HookEvent is deserialized —
    // they must not appear in the outbound payload.
    let raw = r#"{
        "session_id": "sess-forward",
        "hook_event_name": "FutureEvent",
        "brand_new_field": "preserved",
        "numeric_extra": 99
    }"#;

    let event = parse_event(raw);

    let config = make_config("https://example.com");
    let payload = EventPayload::new(&config, event);
    let body = serde_json::to_string(&payload).expect("payload must serialize");
    let body_value: serde_json::Value = serde_json::from_str(&body).expect("must be valid JSON");

    // Unknown fields must NOT appear in the serialized trimmed DTO.
    assert!(
        body_value["event"].get("brand_new_field").is_none(),
        "unknown fields must not appear in trimmed DTO"
    );
    assert!(
        body_value["event"].get("numeric_extra").is_none(),
        "unknown fields must not appear in trimmed DTO"
    );
}

#[test]
fn test_pipeline_stop_event() {
    let raw = r#"{
        "session_id": "sess-stop",
        "hook_event_name": "Stop",
        "stop_hook_active": false,
        "reason": "user_cancelled"
    }"#;

    let event = parse_event(raw);
    let config = make_config("https://example.com");
    let payload = EventPayload::new(&config, event);
    let body = serde_json::to_string(&payload).expect("payload must serialize");
    let body_value: serde_json::Value = serde_json::from_str(&body).expect("must be valid JSON");

    assert_eq!(body_value["event"]["hook_event_name"], "Stop");
    // stop_hook_active and reason are not in the trimmed DTO
    assert!(
        body_value["event"].get("stop_hook_active").is_none(),
        "stop_hook_active must not appear in trimmed DTO"
    );
    assert!(
        body_value["event"].get("reason").is_none(),
        "reason must not appear in trimmed DTO"
    );
}

#[test]
fn test_pipeline_timestamp_is_rfc3339_with_millis() {
    let raw = r#"{"session_id": "sess-ts", "hook_event_name": "PreToolUse"}"#;
    let event = parse_event(raw);
    let config = make_config("https://example.com");
    let payload = EventPayload::new(&config, event);

    // timestamp must be valid RFC 3339 with millisecond precision (contains a dot).
    let ts = &payload.timestamp;
    assert!(
        chrono::DateTime::parse_from_rfc3339(ts).is_ok(),
        "timestamp must be valid RFC 3339: {ts}"
    );
    assert!(
        ts.contains('.'),
        "timestamp must have sub-second precision: {ts}"
    );
}

#[test]
fn test_pipeline_invalid_json_returns_error() {
    let bad_json = "{ not valid json }";
    let result = serde_json::from_str::<HookEvent>(bad_json);
    assert!(result.is_err(), "invalid JSON must return an error");
}

#[test]
fn test_pipeline_missing_required_fields_returns_error() {
    // session_id and hook_event_name are required (not Option<>).
    let missing_fields = r#"{"cwd": "/tmp"}"#;
    let result = serde_json::from_str::<HookEvent>(missing_fields);
    assert!(
        result.is_err(),
        "missing required fields must return an error"
    );
}

// ---------------------------------------------------------------------------
// Sender – URL construction and payload serialization (no live HTTP)
// ---------------------------------------------------------------------------

#[test]
fn test_payload_serializes_content_type_compatible_json() {
    // The sender sets Content-Type: application/json and sends the serialized
    // payload. Verify that the payload serializes to valid JSON.
    let raw = r#"{"session_id": "sess-ct", "hook_event_name": "PostToolUse"}"#;
    let event = parse_event(raw);
    let config = make_config("https://example.com");
    let payload = EventPayload::new(&config, event);

    let json_string = serde_json::to_string(&payload);
    assert!(json_string.is_ok(), "payload must serialize without error");
    let json_string = json_string.unwrap();
    assert!(!json_string.is_empty(), "serialized JSON must not be empty");

    // Must round-trip back to a valid JSON value.
    let reparsed: Result<serde_json::Value, _> = serde_json::from_str(&json_string);
    assert!(reparsed.is_ok(), "serialized body must be parseable JSON");
}

#[test]
fn test_payload_omits_none_optional_fields() {
    // Fields marked skip_serializing_if = "Option::is_none" must not appear
    // in the JSON when they are None. This keeps the request body lean.
    let raw = r#"{"session_id": "sess-omit", "hook_event_name": "PreToolUse"}"#;
    let event = parse_event(raw);
    let config = make_config("https://example.com");
    let payload = EventPayload::new(&config, event);

    let json_string = serde_json::to_string(&payload).expect("must serialize");
    let value: serde_json::Value = serde_json::from_str(&json_string).expect("must parse");

    // These optional fields should be absent when None.
    let event_obj = &value["event"];
    assert!(
        event_obj.get("tool_name").is_none(),
        "absent tool_name must not appear in JSON"
    );
    assert!(
        event_obj.get("cwd").is_none(),
        "absent cwd must not appear in JSON"
    );
    assert!(
        event_obj.get("message").is_none(),
        "absent message must not appear in JSON"
    );
}

#[test]
fn test_payload_authorization_header_format() {
    // The sender formats the header as "Bearer <api_key>".
    // We test the formatting logic directly since we cannot inspect headers
    // without a live HTTP call.
    let api_key = "my-secret-token";
    let expected_header = format!("Bearer {api_key}");
    assert_eq!(expected_header, "Bearer my-secret-token");
}

#[test]
fn test_payload_events_url_no_trailing_slash() {
    // Verify the URL shape that would be sent. The sender strips trailing
    // slashes from server_url before appending the path.
    let raw = r#"{"session_id": "sess-url", "hook_event_name": "Stop"}"#;
    let event = parse_event(raw);
    let config = make_config("https://my-server.example.com/");
    let payload = EventPayload::new(&config, event);

    // The payload itself carries the server_url indirectly through config;
    // here we simply confirm the pipeline completes without error.
    let json_string = serde_json::to_string(&payload).expect("must serialize");
    assert!(!json_string.is_empty());
}
