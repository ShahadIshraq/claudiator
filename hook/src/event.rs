//! Deserialization / serialisation of Claude Code hook events.
//!
//! [`HookEvent`] is both the inbound DTO (deserialized from Claude Code stdin)
//! and the outbound DTO (serialized into the network payload). It contains only
//! the 7 fields the server actually reads. All other fields in the Claude Code
//! JSON payload are silently ignored by serde's default behaviour — no explicit
//! catch-all is needed.

use std::io;

use serde::{Deserialize, Serialize};

use crate::error::EventError;

/// A hook event received from Claude Code and forwarded to the server.
///
/// Only the 7 fields the server reads are declared. Unknown fields in the
/// incoming JSON are silently discarded by serde.
#[derive(Debug, Deserialize, Serialize)]
pub struct HookEvent {
    pub session_id: String,
    pub hook_event_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notification_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl HookEvent {
    /// Parse a [`HookEvent`] from any `Read` source.
    ///
    /// Used by tests to pass a byte slice instead of touching actual stdin.
    pub fn from_reader<R: io::Read>(reader: R) -> Result<Self, EventError> {
        serde_json::from_reader(reader).map_err(EventError::ParseFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_reader_minimal() {
        let json = r#"{"session_id": "sess-1", "hook_event_name": "Stop"}"#;
        let event = HookEvent::from_reader(json.as_bytes()).unwrap();
        assert_eq!(event.session_id, "sess-1");
        assert_eq!(event.hook_event_name, "Stop");
        assert!(event.cwd.is_none());
        assert!(event.tool_name.is_none());
    }

    #[test]
    fn test_from_reader_with_used_fields() {
        let json = r#"{
            "session_id": "sess-2",
            "hook_event_name": "Notification",
            "cwd": "/workspace",
            "prompt": "Go",
            "notification_type": "info",
            "tool_name": "bash",
            "message": "Done"
        }"#;
        let event = HookEvent::from_reader(json.as_bytes()).unwrap();
        assert_eq!(event.cwd, Some("/workspace".to_string()));
        assert_eq!(event.prompt, Some("Go".to_string()));
        assert_eq!(event.notification_type, Some("info".to_string()));
        assert_eq!(event.tool_name, Some("bash".to_string()));
        assert_eq!(event.message, Some("Done".to_string()));
    }

    #[test]
    fn test_from_reader_unknown_fields_silently_dropped() {
        // Claude Code may send many more fields; they must not cause parse errors.
        let json = r#"{
            "session_id": "sess-3",
            "hook_event_name": "PreToolUse",
            "tool_input": {"command": "rm -rf /"},
            "custom_instructions": "secret",
            "transcript_path": "/private/t.json",
            "stop_hook_active": false,
            "extra_future_field": 42
        }"#;
        let event = HookEvent::from_reader(json.as_bytes()).unwrap();
        assert_eq!(event.session_id, "sess-3");
        // Sensitive / unknown fields are not deserialized
        let serialized = serde_json::to_string(&event).unwrap();
        assert!(!serialized.contains("tool_input"));
        assert!(!serialized.contains("custom_instructions"));
        assert!(!serialized.contains("transcript_path"));
    }

    #[test]
    fn test_from_reader_missing_required_fields_errors() {
        let json = r#"{"cwd": "/tmp"}"#;
        assert!(HookEvent::from_reader(json.as_bytes()).is_err());
    }

    #[test]
    fn test_from_reader_invalid_json_errors() {
        assert!(HookEvent::from_reader("{ not json }".as_bytes()).is_err());
    }
}
