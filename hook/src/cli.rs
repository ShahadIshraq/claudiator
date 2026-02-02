use clap::{Parser, Subcommand};

/// Claudiator hook binary — forwards Claude Code events to a remote server
#[derive(Debug, Parser)]
#[command(name = "claudiator-hook", version, about)]
pub struct Cli {
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
        let cli = cli.unwrap();
        assert!(matches!(cli.command, Commands::Send));
    }

    #[test]
    fn test_parse_test_command() {
        let cli = Cli::try_parse_from(["claudiator-hook", "test"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(matches!(cli.command, Commands::Test));
    }

    #[test]
    fn test_parse_version_command() {
        let cli = Cli::try_parse_from(["claudiator-hook", "version"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(matches!(cli.command, Commands::Version));
    }
}
