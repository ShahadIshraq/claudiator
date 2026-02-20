//! The outbound event payload sent to the Claudiator server.
//!
//! [`EventPayload`] wraps the trimmed [`HookEvent`] DTO with device metadata
//! and a server-side timestamp. The server uses the device fields to associate
//! events with a specific registered device, and the timestamp for accurate
//! ordering of events that arrive out of order due to network delays.

use chrono::{SecondsFormat, Utc};
use serde::Serialize;

use crate::config::Config;
use crate::event::{HookEvent, RawHookEvent};

/// Device identity fields included with every event.
///
/// These are copied from [`Config`] at payload-construction time so the server
/// can match events to the correct device without a separate lookup.
#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
}

/// The complete JSON body sent to `POST /api/v1/events`.
#[derive(Debug, Serialize)]
pub struct EventPayload {
    /// Device that produced this event.
    pub device: DeviceInfo,
    /// The trimmed hook event DTO — only server-used fields, no sensitive data.
    pub event: HookEvent,
    /// RFC 3339 timestamp (millisecond precision) of when this payload was created.
    pub timestamp: String,
}

impl EventPayload {
    /// Build a payload from the loaded config and a raw parsed hook event.
    ///
    /// The raw event is converted to the trimmed [`HookEvent`] DTO via
    /// [`From`] before being wrapped, ensuring sensitive fields are dropped
    /// at this boundary.
    pub fn new(config: &Config, raw: RawHookEvent) -> Self {
        let device = DeviceInfo {
            device_id: config.device_id.clone(),
            device_name: config.device_name.clone(),
            platform: config.platform.clone(),
        };

        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

        Self {
            device,
            event: HookEvent::from(raw),
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::RawHookEvent;
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        Config {
            server_url: "https://example.com".to_string(),
            api_key: "test-key".to_string(),
            device_name: "test-machine".to_string(),
            device_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            platform: "mac".to_string(),
            log_level: "error".to_string(),
            max_log_size_bytes: 1_048_576,
            max_log_backups: 2,
        }
    }

    fn create_test_raw_event() -> RawHookEvent {
        RawHookEvent {
            session_id: "sess-123".to_string(),
            hook_event_name: "test_event".to_string(),
            cwd: None,
            transcript_path: None,
            permission_mode: None,
            tool_name: None,
            tool_input: None,
            tool_output: None,
            tool_response: None,
            tool_use_id: None,
            notification_type: None,
            message: None,
            title: None,
            prompt: None,
            source: None,
            model: None,
            stop_hook_active: None,
            reason: None,
            subagent_id: None,
            subagent_type: None,
            agent_id: None,
            agent_type: None,
            agent_transcript_path: None,
            error: None,
            is_interrupt: None,
            teammate_name: None,
            team_name: None,
            task_id: None,
            task_subject: None,
            task_description: None,
            trigger: None,
            custom_instructions: None,
            permission_suggestions: None,
            extra: HashMap::new(),
        }
    }

    #[test]
    fn test_new_payload_device_fields() {
        let config = create_test_config();
        let raw = create_test_raw_event();

        let payload = EventPayload::new(&config, raw);

        assert_eq!(
            payload.device.device_id,
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(payload.device.device_name, "test-machine");
        assert_eq!(payload.device.platform, "mac");
    }

    #[test]
    fn test_new_payload_timestamp_valid_rfc3339() {
        let config = create_test_config();
        let raw = create_test_raw_event();

        let payload = EventPayload::new(&config, raw);

        // Parse the timestamp back to verify it's valid RFC3339
        let parsed = chrono::DateTime::parse_from_rfc3339(&payload.timestamp);
        assert!(
            parsed.is_ok(),
            "Timestamp should be valid RFC3339: {}",
            payload.timestamp
        );

        // Verify it contains milliseconds
        assert!(payload.timestamp.contains('.'));
    }

    #[test]
    fn test_new_payload_event_preserved() {
        let config = create_test_config();
        let raw = create_test_raw_event();
        let original_session_id = raw.session_id.clone();
        let original_event_name = raw.hook_event_name.clone();

        let payload = EventPayload::new(&config, raw);

        assert_eq!(payload.event.session_id, original_session_id);
        assert_eq!(payload.event.hook_event_name, original_event_name);
    }

    #[test]
    fn test_new_payload_drops_sensitive_fields() {
        let config = create_test_config();
        let mut raw = create_test_raw_event();
        raw.tool_input = Some(serde_json::json!({"command": "secret"}));
        raw.custom_instructions = Some("very secret instructions".to_string());
        raw.transcript_path = Some("/private/transcript.json".to_string());

        let payload = EventPayload::new(&config, raw);

        // Serialize and verify sensitive fields are absent
        let json = serde_json::to_string(&payload).unwrap();
        assert!(!json.contains("tool_input"));
        assert!(!json.contains("custom_instructions"));
        assert!(!json.contains("transcript_path"));
        assert!(!json.contains("secret"));
    }
}
