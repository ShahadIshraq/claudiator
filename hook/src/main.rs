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
use event::HookEvent;
use logger::log_error;
use payload::EventPayload;
use sender::{send_event, test_connection};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Send => cmd_send(),
        Commands::Test => cmd_test(),
        Commands::Version => cmd_version(),
    }
}

fn cmd_send() {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            log_error(&format!("Config error: {}", e));
            return;
        }
    };

    let event = match HookEvent::from_stdin() {
        Ok(e) => e,
        Err(e) => {
            log_error(&format!("Event parse error: {}", e));
            return;
        }
    };

    let payload = EventPayload::new(&config, event);

    if let Err(e) = send_event(&config, &payload) {
        log_error(&format!("Send error: {}", e));
    }
}

fn cmd_test() {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    println!("Testing connection to {}...", config.server_url);

    match test_connection(&config) {
        Ok(body) => {
            println!("Connection successful!");
            println!("Server response: {}", body);
        }
        Err(e) => {
            eprintln!("Connection failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_version() {
    println!("claudiator-hook {}", env!("CARGO_PKG_VERSION"));
}
