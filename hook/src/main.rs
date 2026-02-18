//! `claudiator-hook` — a small binary invoked by Claude Code's hook system.
//!
//! Claude Code calls this binary for each hook event (e.g. `PreToolUse`,
//! `PostToolUse`, `Stop`). The binary reads the JSON event from stdin,
//! wraps it with device metadata, and forwards it to the Claudiator server.
//!
//! # Design constraints
//!
//! The hook binary must always exit 0. Claude Code interprets a non-zero exit
//! code as a "block" signal and will surface an error to the user. We never
//! want a backend outage or misconfiguration to disrupt the Claude Code
//! session, so all errors are logged and the process exits cleanly.
//!
//! # Entry point
//!
//! See [`main`] for the top-level dispatch and [`resolve_log_level`] for the
//! log-level precedence rules.

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

use crate::error::ConfigError;
use cli::{Cli, Commands};
use config::Config;
use event::HookEvent;
use logger::{log_debug, log_error, log_info, LogLevel};
use payload::EventPayload;
use sender::{send_event, test_connection};

/// Determine the active log level from all sources.
///
/// Precedence (highest to lowest):
/// 1. `--log-level <LEVEL>` CLI flag
/// 2. `CLAUDIATOR_LOG_LEVEL` environment variable
/// 3. `log_level` field in `~/.claude/claudiator/config.toml`
/// 4. Hard-coded default: `error`
///
/// Invalid values at any tier are silently skipped so the next source
/// can take effect. This avoids a misconfigured env var breaking the hook.
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

    let (config_log_level, max_size, max_backups) =
        config_result
            .as_ref()
            .map_or(("error", 1_048_576, 2), |config| {
                (
                    config.log_level.as_str(),
                    config.max_log_size_bytes,
                    config.max_log_backups,
                )
            });

    let log_level = resolve_log_level(cli.log_level.as_deref(), config_log_level);
    logger::init(log_level, max_size, max_backups);

    match cli.command {
        Commands::Send => cmd_send(config_result),
        Commands::Test => cmd_test(),
        Commands::Version => cmd_version(),
    }
}

/// Handle the `send` subcommand.
///
/// Reads a Claude Code hook event from stdin, wraps it in an [`EventPayload`]
/// containing device metadata, and POSTs it to the server.
///
/// Errors are logged but the function always returns normally so that the
/// process exits 0. A non-zero exit would signal Claude Code to block the
/// current action, which is never the right response to a backend failure.
fn cmd_send(config_result: Result<Config, ConfigError>) {
    let config = match config_result {
        Ok(c) => c,
        Err(e) => {
            log_error(&format!("Config error: {e}"));
            return;
        }
    };

    log_debug(&format!(
        "Processing event for server: {}",
        config.server_url
    ));

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

/// Handle the `test` subcommand.
///
/// Hits the server's `/api/v1/ping` endpoint and prints the result. Unlike
/// `send`, this command exits non-zero on failure — it is only run by the
/// user interactively to verify connectivity, never by Claude Code directly.
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

/// Handle the `version` subcommand.
fn cmd_version() {
    println!("claudiator-hook {}", env!("CARGO_PKG_VERSION"));
}
