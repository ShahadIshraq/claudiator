use std::collections::HashMap;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct EventData {
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

    // Claude Code sends `tool_response` for PostToolUse (distinct from tool_output)
    #[serde(default)]
    pub tool_response: Option<serde_json::Value>,
    #[serde(default)]
    pub tool_use_id: Option<String>,

    #[serde(default)]
    pub notification_type: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub source: Option<String>,

    // Session/model fields
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub stop_hook_active: Option<bool>,

    #[serde(default)]
    pub reason: Option<String>,

    #[serde(default)]
    pub subagent_id: Option<String>,
    #[serde(default)]
    pub subagent_type: Option<String>,

    // Agent fields (distinct from subagent_id/subagent_type)
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub agent_type: Option<String>,
    #[serde(default)]
    pub agent_transcript_path: Option<String>,

    // Error fields
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub is_interrupt: Option<bool>,

    // Team fields
    #[serde(default)]
    pub teammate_name: Option<String>,
    #[serde(default)]
    pub team_name: Option<String>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub task_subject: Option<String>,
    #[serde(default)]
    pub task_description: Option<String>,

    // Compact fields
    #[serde(default)]
    pub trigger: Option<String>,
    #[serde(default)]
    pub custom_instructions: Option<String>,

    // Permission fields
    #[serde(default)]
    pub permission_suggestions: Option<serde_json::Value>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
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
        assert_eq!(payload.event.notification_type, Some("info".to_string()));
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

    #[test]
    fn test_event_payload_tool_response() {
        let json = r#"{
            "device": {
                "device_id": "test-device",
                "device_name": "Test Device",
                "platform": "macos"
            },
            "event": {
                "session_id": "session-1",
                "hook_event_name": "PostToolUse",
                "tool_name": "Bash",
                "tool_input": {"command": "ls"},
                "tool_response": {"stdout": "file1\nfile2"},
                "tool_use_id": "tu-789"
            },
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

        let payload: EventPayload = serde_json::from_str(json).unwrap();
        assert!(payload.event.tool_response.is_some());
        assert_eq!(payload.event.tool_use_id, Some("tu-789".to_string()));
        assert!(payload.event.extra.is_empty());
    }
}
