//! Error types used throughout the hook crate.
//!
//! Each module has its own error enum so callers get precise, typed failure
//! information. All errors implement [`Display`](std::fmt::Display) so they
//! can be formatted into log messages without additional mapping.

use std::io;
use std::path::PathBuf;

/// Errors that can occur while loading `config.toml`.
#[derive(Debug)]
pub enum ConfigError {
    /// The home directory could not be determined from the OS.
    NoHomeDir,
    /// The config file exists but could not be read (e.g. permission denied).
    ReadFailed(PathBuf, io::Error),
    /// The config file was read but is not valid TOML or is missing required fields.
    ParseFailed(PathBuf, toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoHomeDir => write!(f, "Could not determine home directory"),
            Self::ReadFailed(path, err) => {
                write!(f, "Failed to read config file {}: {err}", path.display())
            }
            Self::ParseFailed(path, err) => {
                write!(f, "Failed to parse config file {}: {err}", path.display())
            }
        }
    }
}

/// Errors that can occur while parsing a hook event from stdin.
#[derive(Debug)]
pub enum EventError {
    /// The stdin payload was not valid JSON or did not match the expected shape.
    ParseFailed(serde_json::Error),
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseFailed(err) => write!(f, "Failed to parse event: {err}"),
        }
    }
}

/// Errors that can occur while sending an event to the server.
#[derive(Debug)]
pub enum SendError {
    /// The payload could not be serialized to JSON.
    Serialize(serde_json::Error),
    /// A network-level failure (DNS, connection refused, timeout, etc.).
    Network(String),
    /// The server returned a non-200 HTTP status code.
    ServerError(u16, String),
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(err) => write!(f, "Failed to serialize event: {err}"),
            Self::Network(msg) => write!(f, "Network error: {msg}"),
            Self::ServerError(code, msg) => {
                write!(f, "Server error {code}: {msg}")
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
        let err = ConfigError::ReadFailed(path, io_err);
        let msg = err.to_string();
        assert!(msg.starts_with("Failed to read config file /fake/path:"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_config_error_parse_failed() {
        let path = PathBuf::from("/fake/config.toml");
        let toml_result = toml::from_str::<toml::Value>("invalid toml {{{");
        assert!(toml_result.is_err());
        if let Err(toml_err) = toml_result {
            let err = ConfigError::ParseFailed(path, toml_err);
            let msg = err.to_string();
            assert!(msg.starts_with("Failed to parse config file /fake/config.toml:"));
        }
    }

    #[test]
    fn test_event_error_parse_failed() {
        let json_result = serde_json::from_str::<serde_json::Value>("invalid json {");
        assert!(json_result.is_err());
        if let Err(json_err) = json_result {
            let err = EventError::ParseFailed(json_err);
            let msg = err.to_string();
            assert!(msg.starts_with("Failed to parse event:"));
        }
    }

    #[test]
    fn test_send_error_serialize() {
        // For testing Display, we can just create the error directly
        // using a parse error as a stand-in
        let json_result = serde_json::from_str::<serde_json::Value>("");
        assert!(json_result.is_err());
        if let Err(json_err) = json_result {
            let err = SendError::Serialize(json_err);
            let msg = err.to_string();
            assert!(msg.starts_with("Failed to serialize event:"));
        }
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
