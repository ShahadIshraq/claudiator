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

pub fn send_event(config: &Config, payload: &EventPayload) -> Result<(), SendError> {
    let body = serde_json::to_string(payload).map_err(SendError::Serialize)?;
    let url = build_events_url(&config.server_url);

    let response = ureq::post(&url)
        .timeout(Duration::from_secs(3))
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", config.api_key))
        .set(
            "User-Agent",
            &format!("claudiator-hook/{}", env!("CARGO_PKG_VERSION")),
        )
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

pub fn test_connection(config: &Config) -> Result<String, SendError> {
    let url = build_ping_url(&config.server_url);

    let response = ureq::get(&url)
        .timeout(Duration::from_secs(3))
        .set("Authorization", &format!("Bearer {}", config.api_key))
        .set(
            "User-Agent",
            &format!("claudiator-hook/{}", env!("CARGO_PKG_VERSION")),
        )
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
