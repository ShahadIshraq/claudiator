use std::collections::HashMap;
use std::io;

use serde::{Deserialize, Serialize};

use crate::error::EventError;

#[derive(Debug, Deserialize, Serialize)]
#[allow(clippy::struct_field_names)]
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
    // Claude Code sends `tool_response` for PostToolUse (distinct from tool_output)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_response: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    // Session/model fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_hook_active: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    // Agent fields (distinct from subagent_id/subagent_type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_transcript_path: Option<String>,

    // Error fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_interrupt: Option<bool>,

    // Team fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teammate_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_description: Option<String>,

    // Compact fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_instructions: Option<String>,

    // Permission fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_suggestions: Option<serde_json::Value>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl HookEvent {
    pub fn from_stdin() -> Result<Self, EventError> {
        let stdin = io::stdin();
        let reader = stdin.lock();
        Self::from_reader(reader)
    }

    pub fn from_reader<R: io::Read>(reader: R) -> Result<Self, EventError> {
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

        let event = HookEvent::from_reader(json.as_bytes());
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

        let event = HookEvent::from_reader(json.as_bytes());
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
    fn test_from_reader_minimal_json() {
        let json = r#"{
            "session_id": "sess-123",
            "hook_event_name": "minimal"
        }"#;

        let event = HookEvent::from_reader(json.as_bytes());
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

        let event = HookEvent::from_reader(json.as_bytes());
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

        let event = HookEvent::from_reader(json.as_bytes());
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

        let event = HookEvent::from_reader(json.as_bytes());
        assert!(event.is_ok());
        let Ok(event) = event else { return };

        assert_eq!(event.hook_event_name, "SessionStart");
        assert_eq!(event.model, Some("claude-sonnet-4-5-20250929".to_string()));
        assert_eq!(event.source, Some("vscode".to_string()));
        assert_eq!(event.agent_type, Some("main".to_string()));
        assert!(event.extra.is_empty());
    }
}
