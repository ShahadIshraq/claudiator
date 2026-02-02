use std::io;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConfigError {
    NoHomeDir,
    ReadFailed(PathBuf, io::Error),
    ParseFailed(PathBuf, toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NoHomeDir => write!(f, "Could not determine home directory"),
            ConfigError::ReadFailed(path, err) => {
                write!(f, "Failed to read config file {}: {}", path.display(), err)
            }
            ConfigError::ParseFailed(path, err) => {
                write!(f, "Failed to parse config file {}: {}", path.display(), err)
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum EventError {
    ParseFailed(serde_json::Error),
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventError::ParseFailed(err) => write!(f, "Failed to parse event: {}", err),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum SendError {
    Serialize(serde_json::Error),
    Network(String),
    ServerError(u16, String),
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendError::Serialize(err) => write!(f, "Failed to serialize event: {}", err),
            SendError::Network(msg) => write!(f, "Network error: {}", msg),
            SendError::ServerError(code, msg) => {
                write!(f, "Server error {}: {}", code, msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_config_error_no_home_dir() {
        let err = ConfigError::NoHomeDir;
        assert_eq!(err.to_string(), "Could not determine home directory");
    }

    #[test]
    fn test_config_error_read_failed() {
        let path = PathBuf::from("/fake/path");
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = ConfigError::ReadFailed(path.clone(), io_err);
        let msg = err.to_string();
        assert!(msg.starts_with("Failed to read config file /fake/path:"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_config_error_parse_failed() {
        let path = PathBuf::from("/fake/config.toml");
        let toml_err = toml::from_str::<toml::Value>("invalid toml {{{").unwrap_err();
        let err = ConfigError::ParseFailed(path.clone(), toml_err);
        let msg = err.to_string();
        assert!(msg.starts_with("Failed to parse config file /fake/config.toml:"));
    }

    #[test]
    fn test_event_error_parse_failed() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json {").unwrap_err();
        let err = EventError::ParseFailed(json_err);
        let msg = err.to_string();
        assert!(msg.starts_with("Failed to parse event:"));
    }

    #[test]
    fn test_send_error_serialize() {
        use std::collections::HashMap;

        // Create a Map and then corrupt it to produce a serialization error
        // by attempting to serialize a recursive structure
        let mut map = HashMap::new();
        map.insert("key", "value");

        // For testing Display, we can just create the error directly
        // using a parse error as a stand-in
        let json_err = serde_json::from_str::<serde_json::Value>("").unwrap_err();
        let err = SendError::Serialize(json_err);
        let msg = err.to_string();
        assert!(msg.starts_with("Failed to serialize event:"));
    }

    #[test]
    fn test_send_error_network() {
        let err = SendError::Network("connection timeout".to_string());
        assert_eq!(err.to_string(), "Network error: connection timeout");
    }

    #[test]
    fn test_send_error_server_error() {
        let err = SendError::ServerError(500, "Internal Server Error".to_string());
        assert_eq!(err.to_string(), "Server error 500: Internal Server Error");
    }
}
