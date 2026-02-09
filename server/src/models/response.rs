use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StatusOk {
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_version: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_version: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_version: Option<u64>,
}

impl StatusOk {
    pub(crate) const fn ok() -> Self {
        Self {
            status: "ok",
            server_version: None,
            data_version: None,
            notification_version: None,
        }
    }

    pub(crate) const fn with_version() -> Self {
        Self {
            status: "ok",
            server_version: Some(env!("CARGO_PKG_VERSION")),
            data_version: None,
            notification_version: None,
        }
    }

    pub(crate) const fn with_data_version(v: u64) -> Self {
        Self {
            status: "ok",
            server_version: Some(env!("CARGO_PKG_VERSION")),
            data_version: Some(v),
            notification_version: None,
        }
    }

    pub(crate) const fn with_versions(data_v: u64, notif_v: u64) -> Self {
        Self {
            status: "ok",
            server_version: Some(env!("CARGO_PKG_VERSION")),
            data_version: Some(data_v),
            notification_version: Some(notif_v),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
    pub first_seen: String,
    pub last_seen: String,
    pub active_sessions: i64,
}

#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub devices: Vec<DeviceResponse>,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub device_id: String,
    pub started_at: String,
    pub last_event: String,
    pub status: String,
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionResponse>,
}

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub id: i64,
    pub hook_event_name: String,
    pub timestamp: String,
    pub tool_name: Option<String>,
    pub notification_type: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EventListResponse {
    pub events: Vec<EventResponse>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub event_id: i64,
    pub session_id: String,
    pub device_id: String,
    pub title: String,
    pub body: String,
    pub notification_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationListResponse {
    pub notifications: Vec<NotificationResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_ok_serialization() {
        let status = StatusOk::ok();
        let json = serde_json::to_value(status).unwrap();
        assert_eq!(json["status"], "ok");
        assert!(json["server_version"].is_null());
        assert!(json["data_version"].is_null());
    }

    #[test]
    fn test_status_ok_with_versions() {
        let status = StatusOk::with_versions(42, 100);
        let json = serde_json::to_value(status).unwrap();
        assert_eq!(json["status"], "ok");
        assert!(json["server_version"].is_string());
        assert_eq!(json["data_version"], 42);
        assert_eq!(json["notification_version"], 100);
    }
}
