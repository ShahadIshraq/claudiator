//! Deserialization of Claude Code hook events from stdin.
//!
//! Claude Code writes a JSON object to the hook process's stdin before each
//! invocation. The shape of this object varies by event type (`PreToolUse`,
//! `PostToolUse`, `Stop`, `Notification`, etc.) and may gain new fields as
//! Claude Code evolves.
//!
//! [`RawHookEvent`] covers all known fields as `Option<_>` and uses
//! `#[serde(flatten)]` to capture any unknown fields in [`RawHookEvent::extra`],
//! ensuring forward-compatibility without requiring a hook update.
//!
//! [`HookEvent`] is the trimmed send DTO — only the 7 fields the server
//! actually reads. It is produced from a [`RawHookEvent`] via [`From`] and
//! is the only thing that ever leaves this machine.

use std::collections::HashMap;
use std::io;

use serde::{Deserialize, Serialize};

use crate::error::EventError;

/// A hook event payload emitted by Claude Code, deserialized verbatim from stdin.
///
/// Every field except `session_id` and `hook_event_name` is optional because
/// different event types carry different fields.
///
/// # Forward-compatibility
///
/// The `#[serde(flatten)]` attribute on [`extra`](RawHookEvent::extra) collects
/// any JSON keys that don't match a known field into a `HashMap`. This means
/// the hook will not fail to parse if Claude Code adds new fields in a future
/// version.
///
/// This type is never serialized — it is only read from stdin and converted
/// into the trimmed [`HookEvent`] DTO before transmission.
#[derive(Debug, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct RawHookEvent {
    /// Identifies the Claude Code session that fired this event.
    pub session_id: String,
    /// The name of the hook point, e.g. `"PreToolUse"`, `"Stop"`, `"Notification"`.
    pub hook_event_name: String,

    pub cwd: Option<String>,
    pub transcript_path: Option<String>,
    pub permission_mode: Option<String>,

    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_output: Option<serde_json::Value>,
    pub tool_response: Option<serde_json::Value>,
    pub tool_use_id: Option<String>,

    pub notification_type: Option<String>,
    pub message: Option<String>,
    pub title: Option<String>,

    pub prompt: Option<String>,
    pub source: Option<String>,
    pub model: Option<String>,
    pub stop_hook_active: Option<bool>,

    pub reason: Option<String>,

    pub subagent_id: Option<String>,
    pub subagent_type: Option<String>,
    pub agent_id: Option<String>,
    pub agent_type: Option<String>,
    pub agent_transcript_path: Option<String>,

    pub error: Option<String>,
    pub is_interrupt: Option<bool>,

    pub teammate_name: Option<String>,
    pub team_name: Option<String>,
    pub task_id: Option<String>,
    pub task_subject: Option<String>,
    pub task_description: Option<String>,

    pub trigger: Option<String>,
    pub custom_instructions: Option<String>,

    pub permission_suggestions: Option<serde_json::Value>,

    /// Any JSON fields not matched by a named field above (forward-compat capture).
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl RawHookEvent {
    /// Parse a [`RawHookEvent`] from the process's stdin.
    ///
    /// This is the normal production path: Claude Code pipes the event JSON
    /// to the hook binary's stdin before invoking it.
    pub fn from_stdin() -> Result<Self, EventError> {
        let stdin = io::stdin();
        let reader = stdin.lock();
        Self::from_reader(reader)
    }

    /// Parse a [`RawHookEvent`] from any `Read` source.
    ///
    /// Separated from [`from_stdin`](Self::from_stdin) to allow unit tests to
    /// pass a byte slice instead of touching actual stdin.
    pub fn from_reader<R: io::Read>(reader: R) -> Result<Self, EventError> {
        serde_json::from_reader(reader).map_err(EventError::ParseFailed)
    }
}

/// The trimmed event DTO sent over the wire to the Claudiator server.
///
/// Contains only the 7 fields the server actually reads. All high-sensitivity
/// fields (`tool_input`, `tool_output`, `tool_response`, `custom_instructions`,
/// `transcript_path`, etc.) are intentionally absent — they never leave the
/// client machine.
///
/// Produced from a [`RawHookEvent`] via `HookEvent::from(raw)`.
#[derive(Debug, Serialize)]
pub struct HookEvent {
    pub session_id: String,
    pub hook_event_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl From<RawHookEvent> for HookEvent {
    fn from(raw: RawHookEvent) -> Self {
        Self {
            session_id: raw.session_id,
            hook_event_name: raw.hook_event_name,
            cwd: raw.cwd,
            prompt: raw.prompt,
            notification_type: raw.notification_type,
            tool_name: raw.tool_name,
            message: raw.message,
        }
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

        let event = RawHookEvent::from_reader(json.as_bytes());
        assert!(event.is_ok());
        let Ok(event) = event else { return };

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

        let event = RawHookEvent::from_reader(json.as_bytes());
        assert!(event.is_ok());
        let Ok(event) = event else { return };

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
    #[allow(clippy::cognitive_complexity)]
    fn test_from_reader_minimal_json() {
        let json = r#"{
            "session_id": "sess-123",
            "hook_event_name": "minimal"
        }"#;

        let event = RawHookEvent::from_reader(json.as_bytes());
        assert!(event.is_ok());
        let Ok(event) = event else { return };

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
        assert!(event.tool_response.is_none());
        assert!(event.tool_use_id.is_none());
        assert!(event.title.is_none());
        assert!(event.model.is_none());
        assert!(event.stop_hook_active.is_none());
        assert!(event.agent_id.is_none());
        assert!(event.agent_type.is_none());
        assert!(event.agent_transcript_path.is_none());
        assert!(event.error.is_none());
        assert!(event.is_interrupt.is_none());
        assert!(event.teammate_name.is_none());
        assert!(event.team_name.is_none());
        assert!(event.task_id.is_none());
        assert!(event.task_subject.is_none());
        assert!(event.task_description.is_none());
        assert!(event.trigger.is_none());
        assert!(event.custom_instructions.is_none());
        assert!(event.permission_suggestions.is_none());
        assert!(event.extra.is_empty());
    }

    #[test]
    fn test_from_reader_stop_event() {
        let json = r#"{
            "session_id": "sess-123",
            "hook_event_name": "Stop",
            "stop_hook_active": false
        }"#;

        let event = RawHookEvent::from_reader(json.as_bytes());
        assert!(event.is_ok());
        let Ok(event) = event else { return };

        assert_eq!(event.hook_event_name, "Stop");
        assert_eq!(event.stop_hook_active, Some(false));
        assert!(event.extra.is_empty());
    }

    #[test]
    fn test_from_reader_post_tool_use() {
        let json = r#"{
            "session_id": "sess-123",
            "hook_event_name": "PostToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "ls"},
            "tool_response": {"stdout": "file1\nfile2"},
            "tool_use_id": "tu-789"
        }"#;

        let event = RawHookEvent::from_reader(json.as_bytes());
        assert!(event.is_ok());
        let Ok(event) = event else { return };

        assert_eq!(event.hook_event_name, "PostToolUse");
        assert!(event.tool_response.is_some());
        assert_eq!(event.tool_use_id, Some("tu-789".to_string()));
        assert!(event.extra.is_empty());
    }

    #[test]
    fn test_from_reader_session_start() {
        let json = r#"{
            "session_id": "sess-123",
            "hook_event_name": "SessionStart",
            "model": "claude-sonnet-4-5-20250929",
            "source": "vscode",
            "agent_type": "main",
            "cwd": "/home/user/project",
            "permission_mode": "default"
        }"#;

        let event = RawHookEvent::from_reader(json.as_bytes());
        assert!(event.is_ok());
        let Ok(event) = event else { return };

        assert_eq!(event.hook_event_name, "SessionStart");
        assert_eq!(event.model, Some("claude-sonnet-4-5-20250929".to_string()));
        assert_eq!(event.source, Some("vscode".to_string()));
        assert_eq!(event.agent_type, Some("main".to_string()));
        assert!(event.extra.is_empty());
    }

    #[test]
    fn test_hook_event_from_raw_maps_only_seven_fields() {
        let json = r#"{
            "session_id": "sess-abc",
            "hook_event_name": "Notification",
            "cwd": "/workspace",
            "prompt": "Do the thing",
            "notification_type": "info",
            "tool_name": "bash",
            "message": "All done",
            "tool_input": {"command": "rm -rf /"},
            "custom_instructions": "super secret",
            "transcript_path": "/private/transcript.json"
        }"#;

        let raw = RawHookEvent::from_reader(json.as_bytes()).unwrap();
        let dto = HookEvent::from(raw);

        assert_eq!(dto.session_id, "sess-abc");
        assert_eq!(dto.hook_event_name, "Notification");
        assert_eq!(dto.cwd, Some("/workspace".to_string()));
        assert_eq!(dto.prompt, Some("Do the thing".to_string()));
        assert_eq!(dto.notification_type, Some("info".to_string()));
        assert_eq!(dto.tool_name, Some("bash".to_string()));
        assert_eq!(dto.message, Some("All done".to_string()));

        // Verify the trimmed DTO serializes without sensitive fields
        let serialized = serde_json::to_string(&dto).unwrap();
        assert!(!serialized.contains("tool_input"));
        assert!(!serialized.contains("custom_instructions"));
        assert!(!serialized.contains("transcript_path"));
    }
}
