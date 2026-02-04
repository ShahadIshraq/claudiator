use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "claudiator-server", version, about = "Claudiator event ingestion server")]
pub struct ServerConfig {
    #[arg(long, default_value = "3000", env = "CLAUDIATOR_PORT")]
    pub port: u16,
    #[arg(long, default_value = "claudiator.db", env = "CLAUDIATOR_DB_PATH")]
    pub db_path: String,
    #[arg(long, env = "CLAUDIATOR_API_KEY")]
    pub api_key: String,
    #[arg(long, default_value = "0.0.0.0", env = "CLAUDIATOR_BIND")]
    pub bind: String,
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
