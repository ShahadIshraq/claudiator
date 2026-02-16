use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "claudiator-server",
    version,
    about = "Claudiator event ingestion server"
)]
pub struct ServerConfig {
    #[arg(long, default_value = "3000", env = "CLAUDIATOR_PORT")]
    pub port: u16,
    #[arg(long, default_value = "claudiator.db", env = "CLAUDIATOR_DB_PATH")]
    pub db_path: String,
    #[arg(long, env = "CLAUDIATOR_API_KEY")]
    pub api_key: String,
    #[arg(long, default_value = "0.0.0.0", env = "CLAUDIATOR_BIND")]
    pub bind: String,
    #[arg(long, default_value = "info", env = "CLAUDIATOR_LOG_LEVEL")]
    pub log_level: String,
    #[arg(long, default_value = "logs", env = "CLAUDIATOR_LOG_DIR")]
    pub log_dir: String,
    #[arg(long, env = "CLAUDIATOR_APNS_KEY_PATH")]
    pub apns_key_path: Option<String>,
    #[arg(long, env = "CLAUDIATOR_APNS_KEY_ID")]
    pub apns_key_id: Option<String>,
    #[arg(long, env = "CLAUDIATOR_APNS_TEAM_ID")]
    pub apns_team_id: Option<String>,
    #[arg(long, env = "CLAUDIATOR_APNS_BUNDLE_ID")]
    pub apns_bundle_id: Option<String>,
    #[arg(long, default_value = "false", env = "CLAUDIATOR_APNS_SANDBOX")]
    pub apns_sandbox: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_log_level_is_info() {
        let config = ServerConfig::try_parse_from(["test", "--api-key", "k"]).unwrap();
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn custom_log_level() {
        let config =
            ServerConfig::try_parse_from(["test", "--api-key", "k", "--log-level", "debug"])
                .unwrap();
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn default_log_dir_is_logs() {
        let config = ServerConfig::try_parse_from(["test", "--api-key", "k"]).unwrap();
        assert_eq!(config.log_dir, "logs");
    }

    #[test]
    fn custom_log_dir() {
        let config =
            ServerConfig::try_parse_from(["test", "--api-key", "k", "--log-dir", "/tmp/test-logs"])
                .unwrap();
        assert_eq!(config.log_dir, "/tmp/test-logs");
    }
}
