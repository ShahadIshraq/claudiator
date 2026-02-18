use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Debug, Serialize)]
struct ApnsClaims {
    iss: String,
    iat: u64,
}

struct CachedToken {
    token: String,
    issued_at: u64,
}

#[derive(Debug)]
pub enum ApnsPushResult {
    Success,
    Gone,
    Retry,
    AuthError,
    OtherError(String),
}

pub struct ApnsClient {
    key_id: String,
    team_id: String,
    bundle_id: String,
    signing_key: EncodingKey,
    http_client: reqwest::Client,
    cached_token: RwLock<Option<CachedToken>>,
    default_sandbox: bool,
}

impl ApnsClient {
    #[allow(dead_code)]
    pub(crate) fn new(
        key_path: &str,
        key_id: String,
        team_id: String,
        bundle_id: String,
        default_sandbox: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let key_data = std::fs::read(key_path)?;
        let signing_key = EncodingKey::from_ec_pem(&key_data)?;

        let http_client = reqwest::Client::builder().http2_prior_knowledge().build()?;

        Ok(Self {
            key_id,
            team_id,
            bundle_id,
            signing_key,
            http_client,
            cached_token: RwLock::new(None),
            default_sandbox,
        })
    }

    async fn get_or_refresh_token(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("System time error: {e}"))?
            .as_secs();

        // Check cached token (valid for 50 minutes)
        {
            let cached = self.cached_token.read().await;
            if let Some(ref ct) = *cached {
                if now - ct.issued_at < 3000 {
                    return Ok(ct.token.clone());
                }
            }
        }

        // Generate new token
        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.key_id.clone());

        let claims = ApnsClaims {
            iss: self.team_id.clone(),
            iat: now,
        };

        let token = encode(&header, &claims, &self.signing_key)?;

        // Cache it
        {
            let mut cached = self.cached_token.write().await;
            *cached = Some(CachedToken {
                token: token.clone(),
                issued_at: now,
            });
        }

        Ok(token)
    }

    pub(crate) async fn send_push(
        &self,
        device_token: &str,
        title: &str,
        body: &str,
        collapse_id: Option<&str>,
        notification_id: &str,
        session_id: &str,
        device_id: &str,
        sandbox: bool,
    ) -> ApnsPushResult {
        let token = match self.get_or_refresh_token().await {
            Ok(t) => t,
            Err(e) => return ApnsPushResult::OtherError(format!("Token generation failed: {e}")),
        };

        let host = if sandbox || self.default_sandbox {
            "https://api.sandbox.push.apple.com"
        } else {
            "https://api.push.apple.com"
        };

        let url = format!("{host}/3/device/{device_token}");

        let payload = serde_json::json!({
            "aps": {
                "alert": {
                    "title": title,
                    "body": body,
                },
                "sound": "default",
                "content-available": 1,
            },
            "notification_id": notification_id,
            "session_id": session_id,
            "device_id": device_id,
        });

        let mut request = self
            .http_client
            .post(&url)
            .header("authorization", format!("bearer {token}"))
            .header("apns-topic", &self.bundle_id)
            .header("apns-push-type", "alert")
            .header("apns-priority", "10")
            .json(&payload);

        if let Some(cid) = collapse_id {
            request = request.header("apns-collapse-id", cid);
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let body_text = if matches!(status, 200 | 410 | 403 | 429 | 503) {
                    String::new()
                } else {
                    response.text().await.unwrap_or_default()
                };
                Self::status_to_push_result(status, &body_text)
            }
            Err(e) => ApnsPushResult::OtherError(format!("Request failed: {e}")),
        }
    }

    fn status_to_push_result(status: u16, body: &str) -> ApnsPushResult {
        match status {
            200 => ApnsPushResult::Success,
            410 => ApnsPushResult::Gone,
            403 => ApnsPushResult::AuthError,
            429 | 503 => ApnsPushResult::Retry,
            _ => ApnsPushResult::OtherError(format!("HTTP {status}: {body}")),
        }
    }

    #[cfg(test)]
    fn new_for_test(
        signing_key: EncodingKey,
        key_id: String,
        team_id: String,
        bundle_id: String,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .build()
            .expect("test http client build");
        Self {
            key_id,
            team_id,
            bundle_id,
            signing_key,
            http_client,
            cached_token: RwLock::new(None),
            default_sandbox: true,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

    /// A minimal ES256 (P-256) private key in PKCS8 PEM format used only for tests.
    const TEST_EC_PEM: &[u8] = b"-----BEGIN PRIVATE KEY-----\n\
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgvT5w5eiU5859wGHI\n\
iE0Cu0jnmlsdhPTG/Cur1JBJ2a+hRANCAAR1QTINEESoo+PCsqnLmhFvOCNhbNe5\n\
4qIEjRp8gpuqCEgqZd5RoA8/J1JH/vLZJyv9g1kzxXeCOVOtPolQ9rg6\n\
-----END PRIVATE KEY-----\n";

    fn test_client(key_id: &str, team_id: &str) -> ApnsClient {
        let signing_key = EncodingKey::from_ec_pem(TEST_EC_PEM).expect("valid test EC PEM");
        ApnsClient::new_for_test(
            signing_key,
            key_id.to_string(),
            team_id.to_string(),
            "com.example.test".to_string(),
        )
    }

    // -------------------------------------------------------------------------
    // JWT token structure tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn jwt_has_three_parts() {
        let client = test_client("KEYID12345", "TEAMID6789");
        let token = client
            .get_or_refresh_token()
            .await
            .expect("token generation should succeed");

        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT must have exactly three dot-separated parts");
    }

    #[tokio::test]
    async fn jwt_header_contains_alg_and_kid() {
        let client = test_client("MYKEYID123", "MYTEAMID12");
        let token = client
            .get_or_refresh_token()
            .await
            .expect("token generation should succeed");

        let header_b64 = token.split('.').next().expect("JWT must have header");
        let header_bytes = URL_SAFE_NO_PAD
            .decode(header_b64)
            .expect("header must be valid base64url");
        let header: serde_json::Value =
            serde_json::from_slice(&header_bytes).expect("header must be valid JSON");

        assert_eq!(header["alg"], "ES256", "alg must be ES256");
        assert_eq!(header["kid"], "MYKEYID123", "kid must match key_id");
    }

    #[tokio::test]
    async fn jwt_payload_contains_iss_and_iat() {
        let client = test_client("KEYID00001", "TEAM0000AB");
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let token = client
            .get_or_refresh_token()
            .await
            .expect("token generation should succeed");

        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let parts: Vec<&str> = token.split('.').collect();
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .expect("payload must be valid base64url");
        let payload: serde_json::Value =
            serde_json::from_slice(&payload_bytes).expect("payload must be valid JSON");

        assert_eq!(payload["iss"], "TEAM0000AB", "iss must match team_id");

        let iat = payload["iat"].as_u64().expect("iat must be a number");
        assert!(
            iat >= before && iat <= after,
            "iat ({iat}) must be between before ({before}) and after ({after})"
        );
    }

    // -------------------------------------------------------------------------
    // Token caching tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn token_is_reused_within_cache_window() {
        let client = test_client("KEYID22222", "TEAMID2222");

        let token1 = client
            .get_or_refresh_token()
            .await
            .expect("first token generation should succeed");
        let token2 = client
            .get_or_refresh_token()
            .await
            .expect("second token generation should succeed");

        assert_eq!(token1, token2, "token must be reused within the cache window");
    }

    #[tokio::test]
    async fn token_is_refreshed_after_expiry() {
        let client = test_client("KEYID33333", "TEAMID3333");

        // Seed the cache with a token that is already expired (issued_at = 0)
        {
            let mut cached = client.cached_token.write().await;
            *cached = Some(CachedToken {
                token: "stale.token.value".to_string(),
                issued_at: 0,
            });
        }

        let token = client
            .get_or_refresh_token()
            .await
            .expect("token refresh should succeed");

        assert_ne!(
            token, "stale.token.value",
            "expired cached token must not be returned"
        );
        // The fresh token must still be a valid 3-part JWT
        assert_eq!(
            token.split('.').count(),
            3,
            "refreshed token must be a valid JWT"
        );
    }

    // -------------------------------------------------------------------------
    // Push result parsing tests
    // -------------------------------------------------------------------------

    #[test]
    fn status_200_maps_to_success() {
        assert!(matches!(
            ApnsClient::status_to_push_result(200, ""),
            ApnsPushResult::Success
        ));
    }

    #[test]
    fn status_410_maps_to_gone() {
        assert!(matches!(
            ApnsClient::status_to_push_result(410, ""),
            ApnsPushResult::Gone
        ));
    }

    #[test]
    fn status_403_maps_to_auth_error() {
        assert!(matches!(
            ApnsClient::status_to_push_result(403, ""),
            ApnsPushResult::AuthError
        ));
    }

    #[test]
    fn status_429_maps_to_retry() {
        assert!(matches!(
            ApnsClient::status_to_push_result(429, ""),
            ApnsPushResult::Retry
        ));
    }

    #[test]
    fn status_503_maps_to_retry() {
        assert!(matches!(
            ApnsClient::status_to_push_result(503, ""),
            ApnsPushResult::Retry
        ));
    }

    #[test]
    fn status_500_maps_to_other_error() {
        let result = ApnsClient::status_to_push_result(500, "Internal Server Error");
        match result {
            ApnsPushResult::OtherError(msg) => {
                assert!(msg.contains("500"), "error message must include status code");
                assert!(
                    msg.contains("Internal Server Error"),
                    "error message must include body"
                );
            }
            other => panic!("expected OtherError, got {other:?}"),
        }
    }

    #[test]
    fn status_400_maps_to_other_error_with_body() {
        let result = ApnsClient::status_to_push_result(400, "BadDeviceToken");
        match result {
            ApnsPushResult::OtherError(msg) => {
                assert!(msg.contains("400"));
                assert!(msg.contains("BadDeviceToken"));
            }
            other => panic!("expected OtherError, got {other:?}"),
        }
    }
}
