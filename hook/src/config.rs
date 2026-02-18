//! Configuration loading from `~/.claude/claudiator/config.toml`.
//!
//! The config file is written by the Claudiator server installer and contains
//! the server URL, API key, and device identity. Optional log-related fields
//! have sane defaults so existing configs don't need to be updated.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::ConfigError;

fn default_log_level() -> String {
    "error".to_string()
}

const fn default_max_log_size_bytes() -> u64 {
    1_048_576
}

const fn default_max_log_backups() -> u32 {
    2
}

/// Hook configuration, deserialized from `~/.claude/claudiator/config.toml`.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Base URL of the Claudiator server, e.g. `"https://my-server.example.com"`.
    pub server_url: String,
    /// Bearer token used to authenticate requests to the server.
    pub api_key: String,
    /// Human-readable name for this machine, shown in the server UI.
    pub device_name: String,
    /// Stable UUID identifying this device across reinstalls.
    pub device_id: String,
    /// Host OS platform string (e.g. `"mac"`, `"linux"`).
    pub platform: String,
    /// Minimum severity to write to the log file. Defaults to `"error"`.
    ///
    /// See [`crate::logger::LogLevel`] for accepted values. This can be
    /// overridden at runtime via `CLAUDIATOR_LOG_LEVEL` or `--log-level`.
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Maximum log file size in bytes before rotation. Defaults to 1 MiB.
    #[serde(default = "default_max_log_size_bytes")]
    pub max_log_size_bytes: u64,
    /// Number of rotated log files to retain. Defaults to 2.
    #[serde(default = "default_max_log_backups")]
    pub max_log_backups: u32,
}

impl Config {
    /// Load config from the default path: `~/.claude/claudiator/config.toml`.
    pub fn load() -> Result<Self, ConfigError> {
        let home = dirs::home_dir().ok_or(ConfigError::NoHomeDir)?;
        let path = home.join(".claude").join("claudiator").join("config.toml");
        Self::load_from(&path)
    }

    /// Load config from an explicit path.
    ///
    /// Used by tests to point at a temporary file instead of the real config.
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|err| ConfigError::ReadFailed(path.to_path_buf(), err))?;
        toml::from_str(&content).map_err(|err| ConfigError::ParseFailed(path.to_path_buf(), err))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use tempfile::NamedTempFile;

    const VALID_TOML: &str = r#"
server_url = "https://example.com"
api_key = "test-key-123"
device_name = "test-machine"
device_id = "550e8400-e29b-41d4-a716-446655440000"
platform = "mac"
"#;

    #[test]
    fn test_load_from_valid_toml() {
        let temp_file = NamedTempFile::new();
        assert!(temp_file.is_ok());
        let Ok(mut temp_file) = temp_file else { return };
        let write_result = std::io::Write::write_all(&mut temp_file, VALID_TOML.as_bytes());
        assert!(write_result.is_ok());

        let config = Config::load_from(temp_file.path());
        assert!(config.is_ok());
        if let Ok(config) = config {
            assert_eq!(config.server_url, "https://example.com");
            assert_eq!(config.api_key, "test-key-123");
            assert_eq!(config.device_name, "test-machine");
            assert_eq!(config.device_id, "550e8400-e29b-41d4-a716-446655440000");
            assert_eq!(config.platform, "mac");
            assert_eq!(config.log_level, "error");
            assert_eq!(config.max_log_size_bytes, 1_048_576);
            assert_eq!(config.max_log_backups, 2);
        }
    }

    #[test]
    fn test_load_from_missing_file() {
        let path = PathBuf::from("/nonexistent/path/config.toml");
        let result = Config::load_from(&path);

        assert!(result.is_err());
        if let Err(ConfigError::ReadFailed(p, _)) = result {
            assert_eq!(p, path);
        } else {
            panic!("Expected ReadFailed error");
        }
    }

    #[test]
    fn test_load_from_malformed_toml() {
        let temp_file = NamedTempFile::new();
        assert!(temp_file.is_ok());
        let Ok(mut temp_file) = temp_file else { return };
        let malformed = "invalid toml {{{";
        let write_result = std::io::Write::write_all(&mut temp_file, malformed.as_bytes());
        assert!(write_result.is_ok());

        let result = Config::load_from(temp_file.path());

        assert!(result.is_err());
        if let Err(ConfigError::ParseFailed(p, _)) = result {
            assert_eq!(p, temp_file.path());
        } else {
            panic!("Expected ParseFailed error");
        }
    }

    #[test]
    fn test_load_from_valid_toml_without_new_fields() {
        let temp_file = NamedTempFile::new();
        assert!(temp_file.is_ok());
        let Ok(mut temp_file) = temp_file else { return };
        let write_result = std::io::Write::write_all(&mut temp_file, VALID_TOML.as_bytes());
        assert!(write_result.is_ok());

        let config = Config::load_from(temp_file.path());
        assert!(config.is_ok());
        if let Ok(config) = config {
            assert_eq!(config.log_level, "error");
            assert_eq!(config.max_log_size_bytes, 1_048_576);
            assert_eq!(config.max_log_backups, 2);
        }
    }

    #[test]
    fn test_load_from_valid_toml_with_new_fields() {
        let toml_with_new_fields = r#"
server_url = "https://example.com"
api_key = "test-key-123"
device_name = "test-machine"
device_id = "550e8400-e29b-41d4-a716-446655440000"
platform = "mac"
log_level = "debug"
max_log_size_bytes = 500
max_log_backups = 5
"#;
        let temp_file = NamedTempFile::new();
        assert!(temp_file.is_ok());
        let Ok(mut temp_file) = temp_file else { return };
        let write_result =
            std::io::Write::write_all(&mut temp_file, toml_with_new_fields.as_bytes());
        assert!(write_result.is_ok());

        let config = Config::load_from(temp_file.path());
        assert!(config.is_ok());
        if let Ok(config) = config {
            assert_eq!(config.server_url, "https://example.com");
            assert_eq!(config.api_key, "test-key-123");
            assert_eq!(config.device_name, "test-machine");
            assert_eq!(config.device_id, "550e8400-e29b-41d4-a716-446655440000");
            assert_eq!(config.platform, "mac");
            assert_eq!(config.log_level, "debug");
            assert_eq!(config.max_log_size_bytes, 500);
            assert_eq!(config.max_log_backups, 5);
        }
    }
}
