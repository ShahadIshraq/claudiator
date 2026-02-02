use chrono::{SecondsFormat, Utc};
use serde::Serialize;

use crate::config::Config;
use crate::event::HookEvent;

#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
}

#[derive(Debug, Serialize)]
pub struct EventPayload {
    pub device: DeviceInfo,
    pub event: HookEvent,
    pub timestamp: String,
}

impl EventPayload {
    #[allow(dead_code)]
    pub fn new(config: &Config, event: HookEvent) -> EventPayload {
        let device = DeviceInfo {
            device_id: config.device_id.clone(),
            device_name: config.device_name.clone(),
            platform: config.platform.clone(),
        };

        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

        EventPayload {
            device,
            event,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::HookEvent;
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        Config {
            server_url: "https://example.com".to_string(),
            api_key: "test-key".to_string(),
            device_name: "test-machine".to_string(),
            device_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            platform: "mac".to_string(),
        }
    }

    fn create_test_event() -> HookEvent {
        HookEvent {
            session_id: "sess-123".to_string(),
            hook_event_name: "test_event".to_string(),
            cwd: None,
            transcript_path: None,
            permission_mode: None,
            tool_name: None,
            tool_input: None,
            tool_output: None,
            notification_type: None,
            message: None,
            prompt: None,
            source: None,
            reason: None,
            subagent_id: None,
            subagent_type: None,
            extra: HashMap::new(),
        }
    }

    #[test]
    fn test_new_payload_device_fields() {
        let config = create_test_config();
        let event = create_test_event();

        let payload = EventPayload::new(&config, event);

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
        let event = create_test_event();

        let payload = EventPayload::new(&config, event);

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
        let event = create_test_event();
        let original_session_id = event.session_id.clone();
        let original_event_name = event.hook_event_name.clone();

        let payload = EventPayload::new(&config, event);

        assert_eq!(payload.event.session_id, original_session_id);
        assert_eq!(payload.event.hook_event_name, original_event_name);
    }
}
