use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct EventPayload {
    pub device: DeviceInfo,
    pub event: EventData,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct EventData {
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

    #[serde(default)]
    pub notification_type: Option<String>,
    #[serde(default)]
    pub message: Option<String>,

    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub source: Option<String>,

    #[serde(default)]
    pub reason: Option<String>,

    #[serde(default)]
    pub subagent_id: Option<String>,
    #[serde(default)]
    pub subagent_type: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PushRegisterRequest {
    pub platform: String,
    pub push_token: String,
    #[serde(default)]
    pub sandbox: Option<bool>,
}
