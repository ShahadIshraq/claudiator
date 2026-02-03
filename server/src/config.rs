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
}
