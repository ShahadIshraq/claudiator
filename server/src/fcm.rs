//! Firebase Cloud Messaging (FCM) HTTP v1 API client.

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Debug, Deserialize)]
struct ServiceAccount {
    project_id: String,
    client_email: String,
    private_key: String,
    token_uri: String,
}

#[derive(Debug, Serialize)]
struct GoogleClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: u64,
    exp: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_in: u64,
}

struct CachedToken {
    access_token: String,
    expires_at: u64,
}

/// Outcome of an FCM push attempt.
#[derive(Debug)]
pub enum FcmPushResult {
    /// Push delivered successfully.
    Success,
    /// The device token is no longer valid and should be removed.
    InvalidToken,
    /// Transient failure — retry later.
    Retry,
    /// Authentication / authorization problem with our credentials.
    AuthError,
    /// Any other error.
    OtherError(String),
}

/// Client for the FCM HTTP v1 API backed by a Google service account.
pub struct FcmClient {
    project_id: String,
    client_email: String,
    token_uri: String,
    signing_key: EncodingKey,
    http_client: reqwest::Client,
    cached_token: RwLock<Option<CachedToken>>,
}

impl FcmClient {
    /// Build an `FcmClient` from a Google service-account JSON key file.
    pub fn from_service_account_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let sa: ServiceAccount = serde_json::from_str(&data)?;
        let signing_key = EncodingKey::from_rsa_pem(sa.private_key.as_bytes())?;
        let http_client = reqwest::Client::new();

        Ok(Self {
            project_id: sa.project_id,
            client_email: sa.client_email,
            token_uri: sa.token_uri,
            signing_key,
            http_client,
            cached_token: RwLock::new(None),
        })
    }

    async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("System time error: {e}"))?
            .as_secs();

        // Check cache (refresh 5 minutes before expiry)
        {
            let cached = self.cached_token.read().await;
            if let Some(ref ct) = *cached {
                if now + 300 < ct.expires_at {
                    return Ok(ct.access_token.clone());
                }
            }
        }

        // Generate JWT for Google OAuth2
        let claims = GoogleClaims {
            iss: self.client_email.clone(),
            scope: "https://www.googleapis.com/auth/firebase.messaging".to_string(),
            aud: self.token_uri.clone(),
            iat: now,
            exp: now + 3600,
        };

        let header = Header::new(Algorithm::RS256);
        let jwt = encode(&header, &claims, &self.signing_key)?;

        // Exchange JWT for access token
        let response = self
            .http_client
            .post(&self.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| format!("Token exchange request failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Token exchange failed: HTTP {status}: {body}").into());
        }

        let token_resp: TokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {e}"))?;

        // Cache
        {
            let mut cached = self.cached_token.write().await;
            *cached = Some(CachedToken {
                access_token: token_resp.access_token.clone(),
                expires_at: now + token_resp.expires_in,
            });
        }

        Ok(token_resp.access_token)
    }

    /// Send a data-only push notification via FCM.
    pub async fn send_push(
        &self,
        device_token: &str,
        title: &str,
        body: &str,
        notification_id: &str,
        session_id: &str,
        device_id: &str,
    ) -> FcmPushResult {
        let access_token = match self.get_access_token().await {
            Ok(t) => t,
            Err(e) => return FcmPushResult::OtherError(format!("Token error: {e}")),
        };

        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.project_id
        );

        // Use data-only message (no "notification" block) so onMessageReceived
        // always fires, giving the app full control over display and dedup.
        let payload = serde_json::json!({
            "message": {
                "token": device_token,
                "data": {
                    "notification_id": notification_id,
                    "session_id": session_id,
                    "device_id": device_id,
                    "title": title,
                    "body": body,
                },
                "android": {
                    "priority": "high"
                }
            }
        });

        match self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status().as_u16();
                let body_text = response.text().await.unwrap_or_default();
                Self::status_to_push_result(status, &body_text)
            }
            Err(e) => FcmPushResult::OtherError(format!("Request failed: {e}")),
        }
    }

    fn status_to_push_result(status: u16, body: &str) -> FcmPushResult {
        match status {
            200 => FcmPushResult::Success,
            400 | 404 => {
                // INVALID_ARGUMENT or NOT_FOUND — token is bad
                if body.contains("UNREGISTERED")
                    || body.contains("NOT_FOUND")
                    || body.contains("INVALID_ARGUMENT")
                {
                    FcmPushResult::InvalidToken
                } else {
                    FcmPushResult::OtherError(format!("HTTP {status}: {body}"))
                }
            }
            401 | 403 => FcmPushResult::AuthError,
            429 | 503 => FcmPushResult::Retry,
            _ => FcmPushResult::OtherError(format!("HTTP {status}: {body}")),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Push result parsing tests
    // -------------------------------------------------------------------------

    #[test]
    fn status_200_maps_to_success() {
        assert!(matches!(
            FcmClient::status_to_push_result(200, ""),
            FcmPushResult::Success
        ));
    }

    #[test]
    fn status_400_unregistered_maps_to_invalid_token() {
        assert!(matches!(
            FcmClient::status_to_push_result(400, r#"{"error":{"status":"UNREGISTERED"}}"#),
            FcmPushResult::InvalidToken
        ));
    }

    #[test]
    fn status_404_not_found_maps_to_invalid_token() {
        assert!(matches!(
            FcmClient::status_to_push_result(404, r#"{"error":{"status":"NOT_FOUND"}}"#),
            FcmPushResult::InvalidToken
        ));
    }

    #[test]
    fn status_400_invalid_argument_maps_to_invalid_token() {
        assert!(matches!(
            FcmClient::status_to_push_result(400, r#"{"error":{"status":"INVALID_ARGUMENT"}}"#),
            FcmPushResult::InvalidToken
        ));
    }

    #[test]
    fn status_400_other_maps_to_other_error() {
        let result = FcmClient::status_to_push_result(400, "something else");
        assert!(matches!(result, FcmPushResult::OtherError(_)));
    }

    #[test]
    fn status_401_maps_to_auth_error() {
        assert!(matches!(
            FcmClient::status_to_push_result(401, ""),
            FcmPushResult::AuthError
        ));
    }

    #[test]
    fn status_403_maps_to_auth_error() {
        assert!(matches!(
            FcmClient::status_to_push_result(403, ""),
            FcmPushResult::AuthError
        ));
    }

    #[test]
    fn status_429_maps_to_retry() {
        assert!(matches!(
            FcmClient::status_to_push_result(429, ""),
            FcmPushResult::Retry
        ));
    }

    #[test]
    fn status_503_maps_to_retry() {
        assert!(matches!(
            FcmClient::status_to_push_result(503, ""),
            FcmPushResult::Retry
        ));
    }

    #[test]
    fn status_500_maps_to_other_error() {
        let result = FcmClient::status_to_push_result(500, "Internal Server Error");
        match result {
            FcmPushResult::OtherError(msg) => {
                assert!(
                    msg.contains("500"),
                    "error message must include status code"
                );
                assert!(
                    msg.contains("Internal Server Error"),
                    "error message must include body"
                );
            }
            other => panic!("expected OtherError, got {other:?}"),
        }
    }
}
