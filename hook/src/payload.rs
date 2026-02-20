//! The outbound event payload sent to the Claudiator server.
//!
//! [`EventPayload`] wraps a [`HookEvent`] with device metadata and a
//! timestamp. The server uses the device fields to associate events with a
//! specific registered device.

use chrono::{SecondsFormat, Utc};
use serde::Serialize;

use crate::config::Config;
use crate::event::HookEvent;

/// Device identity fields included with every event.
#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
}

/// The complete JSON body sent to `POST /api/v1/events`.
#[derive(Debug, Serialize)]
pub struct EventPayload {
    pub device: DeviceInfo,
    pub event: HookEvent,
    /// RFC 3339 timestamp (millisecond precision) of when this payload was created.
    pub timestamp: String,
}

impl EventPayload {
    /// Build a payload from the loaded config and a parsed hook event.
    pub fn new(config: &Config, event: HookEvent) -> Self {
        let device = DeviceInfo {
            device_id: config.device_id.clone(),
            device_name: config.device_name.clone(),
            platform: config.platform.clone(),
        };
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        Self {
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

    fn make_config() -> Config {
        Config {
            server_url: "https://example.com".to_string(),
            api_key: "test-key".to_string(),
            device_name: "test-machine".to_string(),
            device_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            platform: "mac".to_string(),
            log_level: "error".to_string(),
            max_log_size_bytes: 1_048_576,
            max_log_backups: 2,
            raw_event_log_path: None,
        }
    }

    fn make_event() -> HookEvent {
        HookEvent {
            session_id: "sess-123".to_string(),
            hook_event_name: "test_event".to_string(),
            cwd: None,
            prompt: None,
            notification_type: None,
            tool_name: None,
            message: None,
        }
    }

    #[test]
    fn test_device_fields() {
        let payload = EventPayload::new(&make_config(), make_event());
        assert_eq!(
            payload.device.device_id,
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(payload.device.device_name, "test-machine");
        assert_eq!(payload.device.platform, "mac");
    }

    #[test]
    fn test_timestamp_valid_rfc3339_with_millis() {
        let payload = EventPayload::new(&make_config(), make_event());
        assert!(chrono::DateTime::parse_from_rfc3339(&payload.timestamp).is_ok());
        assert!(payload.timestamp.contains('.'));
    }

    #[test]
    fn test_event_fields_preserved() {
        let payload = EventPayload::new(&make_config(), make_event());
        assert_eq!(payload.event.session_id, "sess-123");
        assert_eq!(payload.event.hook_event_name, "test_event");
    }
}
