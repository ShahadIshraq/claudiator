use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct EventPayload {
    pub device: DeviceInfo,
    pub event: EventData,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct EventData {
    pub session_id: String,
    pub hook_event_name: String,

    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub permission_mode: Option<String>,

    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub tool_output: Option<serde_json::Value>,

    #[serde(default)]
    pub notification_type: Option<String>,
    #[serde(default)]
    pub message: Option<String>,

    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub source: Option<String>,

    #[serde(default)]
    pub reason: Option<String>,

    #[serde(default)]
    pub subagent_id: Option<String>,
    #[serde(default)]
    pub subagent_type: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PushRegisterRequest {
    pub platform: String,
    pub push_token: String,
    #[serde(default)]
    pub sandbox: Option<bool>,
}

#[cfg(test)]
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
    fn test_event_payload_full() {
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
                "tool_input": {"command": "ls"},
                "notification_type": "info",
                "message": "Running command"
            },
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

        let payload: EventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.event.cwd, Some("/home/user".to_string()));
        assert_eq!(payload.event.tool_name, Some("bash".to_string()));
        assert_eq!(
            payload.event.notification_type,
            Some("info".to_string())
        );
        assert!(payload.event.tool_input.is_some());
    }

    #[test]
    fn test_event_payload_unknown_fields() {
        let json = r#"{
            "device": {
                "device_id": "test-device",
                "device_name": "Test Device",
                "platform": "macos"
            },
            "event": {
                "session_id": "session-1",
                "hook_event_name": "custom-event",
                "unknown_field": "should be captured",
                "another_unknown": 123
            },
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

        let payload: EventPayload = serde_json::from_str(json).unwrap();
        assert!(payload.event.extra.contains_key("unknown_field"));
        assert!(payload.event.extra.contains_key("another_unknown"));
    }
}
