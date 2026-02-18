//! Command-line interface definition for `claudiator-hook`.
//!
//! Parsed once at startup by [`clap`]. The resolved [`Cli`] is then used by
//! `main` to choose the subcommand and (optionally) override the log level.

use clap::{Parser, Subcommand};

/// Claudiator hook binary — forwards Claude Code events to a remote server
#[derive(Debug, Parser)]
#[command(name = "claudiator-hook", version, about)]
pub struct Cli {
    /// Override the log level for this invocation.
    ///
    /// Accepts `error`, `warn`, `info`, or `debug` (case-insensitive).
    /// Takes precedence over `CLAUDIATOR_LOG_LEVEL` and `config.toml`.
    #[arg(long, global = true)]
    pub log_level: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Read a hook event from stdin and send it to the server
    Send,
    /// Test the connection to the configured server
    Test,
    /// Print the version and exit
    Version,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_send_command() {
        let cli = Cli::try_parse_from(["claudiator-hook", "send"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(matches!(cli.command, Commands::Send));
        }
    }

    #[test]
    fn test_parse_test_command() {
        let cli = Cli::try_parse_from(["claudiator-hook", "test"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(matches!(cli.command, Commands::Test));
        }
    }

    #[test]
    fn test_parse_version_command() {
        let cli = Cli::try_parse_from(["claudiator-hook", "version"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(matches!(cli.command, Commands::Version));
        }
    }

    #[test]
    fn test_parse_without_log_level() {
        let cli = Cli::try_parse_from(["claudiator-hook", "send"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(cli.log_level.is_none());
        }
    }

    #[test]
    fn test_parse_with_log_level_before_subcommand() {
        let cli = Cli::try_parse_from(["claudiator-hook", "--log-level", "debug", "send"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert_eq!(cli.log_level, Some("debug".to_string()));
        }
    }

    #[test]
    fn test_parse_with_log_level_after_subcommand() {
        let cli = Cli::try_parse_from(["claudiator-hook", "send", "--log-level", "info"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert_eq!(cli.log_level, Some("info".to_string()));
        }
    }
}
