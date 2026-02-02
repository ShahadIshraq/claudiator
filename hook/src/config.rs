use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::ConfigError;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub api_key: String,
    pub device_name: String,
    pub device_id: String,
    pub platform: String,
}

impl Config {
    pub fn load() -> Result<Config, ConfigError> {
        let home = dirs::home_dir().ok_or(ConfigError::NoHomeDir)?;
        let path = home.join(".claude").join("claudiator").join("config.toml");
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Config, ConfigError> {
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
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, VALID_TOML.as_bytes()).unwrap();

        let config = Config::load_from(temp_file.path()).unwrap();

        assert_eq!(config.server_url, "https://example.com");
        assert_eq!(config.api_key, "test-key-123");
        assert_eq!(config.device_name, "test-machine");
        assert_eq!(config.device_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(config.platform, "mac");
    }

    #[test]
    fn test_load_from_missing_file() {
        let path = PathBuf::from("/nonexistent/path/config.toml");
        let result = Config::load_from(&path);

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ReadFailed(p, _) => {
                assert_eq!(p, path);
            }
            _ => panic!("Expected ReadFailed error"),
        }
    }

    #[test]
    fn test_load_from_malformed_toml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let malformed = "invalid toml {{{";
        std::io::Write::write_all(&mut temp_file, malformed.as_bytes()).unwrap();

        let result = Config::load_from(temp_file.path());

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ParseFailed(p, _) => {
                assert_eq!(p, temp_file.path());
            }
            _ => panic!("Expected ParseFailed error"),
        }
    }
}
