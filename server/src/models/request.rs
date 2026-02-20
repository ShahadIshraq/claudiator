use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct EventPayload {
    pub device: DeviceInfo,
    pub event: EventData,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
}

/// Inbound event data from the hook binary.
///
/// Contains only the 7 fields the server actually reads. Unknown fields in the
/// incoming JSON are silently ignored by serde — no `extra` catch-all needed.
/// This also means `event_json` stored in the database will only contain these
/// 7 fields and never any sensitive data.
#[derive(Debug, Deserialize, Serialize)]
pub struct EventData {
    pub session_id: String,
    pub hook_event_name: String,

    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub notification_type: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PushRegisterRequest {
    pub platform: String,
    pub push_token: String,
    #[serde(default)]
    pub sandbox: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AckRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub scopes: Vec<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_event_payload_minimal() {
        let json = r#"{
            "device": {
                "device_id": "test-device",
                "device_name": "Test Device",
                "platform": "macos"
            },
            "event": {
                "session_id": "session-1",
                "hook_event_name": "session-start"
            },
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

        let payload: EventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.device.device_id, "test-device");
        assert_eq!(payload.event.session_id, "session-1");
        assert_eq!(payload.event.hook_event_name, "session-start");
        assert!(payload.event.cwd.is_none());
        assert!(payload.event.tool_name.is_none());
    }

    #[test]
    fn test_event_payload_with_used_fields() {
        let json = r#"{
            "device": {
                "device_id": "test-device",
                "device_name": "Test Device",
                "platform": "macos"
            },
            "event": {
                "session_id": "session-1",
                "hook_event_name": "tool-use",
                "cwd": "/home/user",
                "tool_name": "bash",
                "notification_type": "info",
                "message": "Running command",
                "prompt": "Do the thing"
            },
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

        let payload: EventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.event.cwd, Some("/home/user".to_string()));
        assert_eq!(payload.event.tool_name, Some("bash".to_string()));
        assert_eq!(payload.event.notification_type, Some("info".to_string()));
        assert_eq!(payload.event.message, Some("Running command".to_string()));
        assert_eq!(payload.event.prompt, Some("Do the thing".to_string()));
    }

    #[test]
    fn test_event_payload_unknown_fields_silently_dropped() {
        // Unknown fields (tool_input, custom_instructions, etc.) must not
        // cause a parse error — serde ignores them by default.
        let json = r#"{
            "device": {
                "device_id": "test-device",
                "device_name": "Test Device",
                "platform": "macos"
            },
            "event": {
                "session_id": "session-1",
                "hook_event_name": "custom-event",
                "tool_input": {"command": "ls"},
                "custom_instructions": "secret",
                "transcript_path": "/tmp/t.json",
                "unknown_field": "should be dropped",
                "another_unknown": 123
            },
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

        let payload: EventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.event.session_id, "session-1");
        // All unknown fields silently dropped — no error
    }

    #[test]
    fn test_event_data_serializes_only_known_fields() {
        let data = EventData {
            session_id: "s1".to_string(),
            hook_event_name: "Stop".to_string(),
            cwd: Some("/workspace".to_string()),
            prompt: None,
            notification_type: None,
            tool_name: Some("bash".to_string()),
            message: None,
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("session_id"));
        assert!(json.contains("cwd"));
        assert!(json.contains("tool_name"));
        // Must not contain any sensitive field names
        assert!(!json.contains("tool_input"));
        assert!(!json.contains("tool_response"));
        assert!(!json.contains("custom_instructions"));
    }
}
