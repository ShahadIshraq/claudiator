use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StatusOk {
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_version: Option<&'static str>,
}

impl StatusOk {
    pub fn ok() -> Self {
        StatusOk {
            status: "ok",
            server_version: None,
        }
    }

    pub fn with_version() -> Self {
        StatusOk {
            status: "ok",
            server_version: Some(env!("CARGO_PKG_VERSION")),
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
