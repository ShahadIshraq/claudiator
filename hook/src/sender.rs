//! HTTP transport for forwarding events to the Claudiator server.
//!
//! All requests use a hard-coded 3-second timeout. The hook is invoked
//! synchronously by Claude Code on every hook event, so a slow or unreachable
//! server must not stall the Claude Code session.

use std::time::Duration;

use crate::config::Config;
use crate::error::SendError;
use crate::payload::EventPayload;

fn build_events_url(server_url: &str) -> String {
    format!("{}/api/v1/events", server_url.trim_end_matches('/'))
}

fn build_ping_url(server_url: &str) -> String {
    format!("{}/api/v1/ping", server_url.trim_end_matches('/'))
}

/// POST a hook event payload to `POST /api/v1/events`.
///
/// Authenticates with a `Bearer` token from the config and includes a
/// `User-Agent` header for server-side diagnostics. Returns `Ok(())` only
/// for HTTP 200; any other status is returned as [`SendError::ServerError`].
pub fn send_event(config: &Config, payload: &EventPayload) -> Result<(), SendError> {
    let body = serde_json::to_string(payload).map_err(SendError::Serialize)?;
    let url = build_events_url(&config.server_url);

    let api_key = &config.api_key;
    let version = env!("CARGO_PKG_VERSION");
    let response = ureq::post(&url)
        .timeout(Duration::from_secs(3))
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {api_key}"))
        .set("User-Agent", &format!("claudiator-hook/{version}"))
        .send_string(&body);

    match response {
        Ok(resp) => {
            if resp.status() == 200 {
                Ok(())
            } else {
                let status = resp.status();
                let body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                Err(SendError::ServerError(status, body))
            }
        }
        Err(ureq::Error::Status(code, response)) => {
            let body = response
                .into_string()
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            Err(SendError::ServerError(code, body))
        }
        Err(err) => Err(SendError::Network(err.to_string())),
    }
}

/// GET `/api/v1/ping` and return the response body as a string.
///
/// Used by the `test` subcommand to verify the server is reachable and the
/// API key is valid before configuring hooks in Claude Code.
pub fn test_connection(config: &Config) -> Result<String, SendError> {
    let url = build_ping_url(&config.server_url);

    let api_key = &config.api_key;
    let version = env!("CARGO_PKG_VERSION");
    let response = ureq::get(&url)
        .timeout(Duration::from_secs(3))
        .set("Authorization", &format!("Bearer {api_key}"))
        .set("User-Agent", &format!("claudiator-hook/{version}"))
        .call();

    match response {
        Ok(resp) => {
            if resp.status() == 200 {
                let body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                Ok(body)
            } else {
                let status = resp.status();
                let body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                Err(SendError::ServerError(status, body))
            }
        }
        Err(ureq::Error::Status(code, response)) => {
            let body = response
                .into_string()
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            Err(SendError::ServerError(code, body))
        }
        Err(err) => Err(SendError::Network(err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_events_url() {
        assert_eq!(
            build_events_url("https://example.com"),
            "https://example.com/api/v1/events"
        );
        assert_eq!(
            build_events_url("https://example.com/"),
            "https://example.com/api/v1/events"
        );
        assert_eq!(
            build_events_url("https://example.com///"),
            "https://example.com/api/v1/events"
        );
    }

    #[test]
    fn test_build_ping_url() {
        assert_eq!(
            build_ping_url("https://example.com"),
            "https://example.com/api/v1/ping"
        );
        assert_eq!(
            build_ping_url("https://example.com/"),
            "https://example.com/api/v1/ping"
        );
        assert_eq!(
            build_ping_url("https://example.com///"),
            "https://example.com/api/v1/ping"
        );
    }
}
