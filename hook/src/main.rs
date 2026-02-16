#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::cargo)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]

mod cli;
mod config;
mod error;
mod event;
mod logger;
mod payload;
mod sender;

use clap::Parser;

use cli::{Cli, Commands};
use config::Config;
use crate::error::ConfigError;
use event::HookEvent;
use logger::{log_error, log_info, log_debug, LogLevel};
use payload::EventPayload;
use sender::{send_event, test_connection};

fn resolve_log_level(cli_level: Option<&str>, config_level: &str) -> LogLevel {
    // Precedence: CLI flag > env var > config > default (Error)
    if let Some(level_str) = cli_level {
        if let Ok(level) = level_str.parse::<LogLevel>() {
            return level;
        }
    }

    if let Ok(env_level) = std::env::var("CLAUDIATOR_LOG_LEVEL") {
        if let Ok(level) = env_level.parse::<LogLevel>() {
            return level;
        }
    }

    if let Ok(level) = config_level.parse::<LogLevel>() {
        return level;
    }

    LogLevel::Error
}

fn main() {
    let cli = Cli::parse();

    let config_result = Config::load();

    let (config_log_level, max_size, max_backups) = config_result.as_ref().map_or(("error", 1_048_576, 2), |config| (
            config.log_level.as_str(),
            config.max_log_size_bytes,
            config.max_log_backups,
        ));

    let log_level = resolve_log_level(cli.log_level.as_deref(), config_log_level);
    logger::init(log_level, max_size, max_backups);

    match cli.command {
        Commands::Send => cmd_send(config_result),
        Commands::Test => cmd_test(),
        Commands::Version => cmd_version(),
    }
}

fn cmd_send(config_result: Result<Config, ConfigError>) {
    let config = match config_result {
        Ok(c) => c,
        Err(e) => {
            log_error(&format!("Config error: {e}"));
            return;
        }
    };

    log_debug(&format!("Processing event for server: {}", config.server_url));

    let event = match HookEvent::from_stdin() {
        Ok(e) => e,
        Err(e) => {
            log_error(&format!("Event parse error: {e}"));
            return;
        }
    };

    let payload = EventPayload::new(&config, event);

    if let Err(e) = send_event(&config, &payload) {
        log_error(&format!("Send error: {e}"));
    } else {
        log_info("Event sent successfully");
    }
}

fn cmd_test() {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            std::process::exit(1);
        }
    };

    println!("Testing connection to {}...", config.server_url);

    match test_connection(&config) {
        Ok(body) => {
            println!("Connection successful!");
            println!("Server response: {body}");
        }
        Err(e) => {
            eprintln!("Connection failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_version() {
    println!("claudiator-hook {}", env!("CARGO_PKG_VERSION"));
}
