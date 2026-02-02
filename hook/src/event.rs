use std::collections::HashMap;
use std::io;

use serde::{Deserialize, Serialize};

use crate::error::EventError;

#[derive(Debug, Deserialize, Serialize)]
pub struct HookEvent {
    pub session_id: String,
    pub hook_event_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_output: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl HookEvent {
    pub fn from_stdin() -> Result<HookEvent, EventError> {
        let stdin = io::stdin();
        let reader = stdin.lock();
        Self::from_reader(reader)
    }

    pub fn from_reader<R: io::Read>(reader: R) -> Result<HookEvent, EventError> {
        serde_json::from_reader(reader).map_err(EventError::ParseFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_reader_full_notification() {
        let json = r#"{
            "session_id": "sess-123",
            "hook_event_name": "notification",
            "cwd": "/home/user/project",
            "transcript_path": "/tmp/transcript.json",
            "permission_mode": "auto",
            "tool_name": "bash",
            "tool_input": {"command": "ls"},
            "tool_output": {"result": "file1\nfile2"},
            "notification_type": "info",
            "message": "Operation complete",
            "prompt": "Continue?",
            "source": "user",
            "reason": "test",
            "subagent_id": "agent-456",
            "subagent_type": "coder"
        }"#;

        let event = HookEvent::from_reader(json.as_bytes()).unwrap();

        assert_eq!(event.session_id, "sess-123");
        assert_eq!(event.hook_event_name, "notification");
        assert_eq!(event.cwd, Some("/home/user/project".to_string()));
        assert_eq!(
            event.transcript_path,
            Some("/tmp/transcript.json".to_string())
        );
        assert_eq!(event.permission_mode, Some("auto".to_string()));
        assert_eq!(event.tool_name, Some("bash".to_string()));
        assert!(event.tool_input.is_some());
        assert!(event.tool_output.is_some());
        assert_eq!(event.notification_type, Some("info".to_string()));
        assert_eq!(event.message, Some("Operation complete".to_string()));
        assert_eq!(event.prompt, Some("Continue?".to_string()));
        assert_eq!(event.source, Some("user".to_string()));
        assert_eq!(event.reason, Some("test".to_string()));
        assert_eq!(event.subagent_id, Some("agent-456".to_string()));
        assert_eq!(event.subagent_type, Some("coder".to_string()));
        assert!(event.extra.is_empty());
    }

    #[test]
    fn test_from_reader_with_unknown_fields() {
        let json = r#"{
            "session_id": "sess-123",
            "hook_event_name": "custom",
            "custom_field": "custom_value",
            "another_unknown": 42
        }"#;

        let event = HookEvent::from_reader(json.as_bytes()).unwrap();

        assert_eq!(event.session_id, "sess-123");
        assert_eq!(event.hook_event_name, "custom");
        assert_eq!(event.extra.len(), 2);
        assert_eq!(
            event.extra.get("custom_field"),
            Some(&serde_json::Value::String("custom_value".to_string()))
        );
        assert_eq!(
            event.extra.get("another_unknown"),
            Some(&serde_json::Value::Number(42.into()))
        );
    }

    #[test]
    fn test_from_reader_minimal_json() {
        let json = r#"{
            "session_id": "sess-123",
            "hook_event_name": "minimal"
        }"#;

        let event = HookEvent::from_reader(json.as_bytes()).unwrap();

        assert_eq!(event.session_id, "sess-123");
        assert_eq!(event.hook_event_name, "minimal");
        assert!(event.cwd.is_none());
        assert!(event.transcript_path.is_none());
        assert!(event.permission_mode.is_none());
        assert!(event.tool_name.is_none());
        assert!(event.tool_input.is_none());
        assert!(event.tool_output.is_none());
        assert!(event.notification_type.is_none());
        assert!(event.message.is_none());
        assert!(event.prompt.is_none());
        assert!(event.source.is_none());
        assert!(event.reason.is_none());
        assert!(event.subagent_id.is_none());
        assert!(event.subagent_type.is_none());
        assert!(event.extra.is_empty());
    }
}
