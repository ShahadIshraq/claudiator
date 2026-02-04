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
    pub fn new(
        key_path: &str,
        key_id: String,
        team_id: String,
        bundle_id: String,
        default_sandbox: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let key_data = std::fs::read(key_path)?;
        let signing_key = EncodingKey::from_ec_pem(&key_data)?;

        let http_client = reqwest::Client::builder()
            .http2_prior_knowledge()
            .build()?;

        Ok(ApnsClient {
            key_id,
            team_id,
            bundle_id,
            signing_key,
            http_client,
            cached_token: RwLock::new(None),
            default_sandbox,
        })
    }

    async fn get_or_refresh_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
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

    pub async fn send_push(
        &self,
        device_token: &str,
        title: &str,
        body: &str,
        collapse_id: Option<&str>,
        sandbox: bool,
    ) -> ApnsPushResult {
        let token = match self.get_or_refresh_token().await {
            Ok(t) => t,
            Err(e) => return ApnsPushResult::OtherError(format!("Token generation failed: {}", e)),
        };

        let host = if sandbox || self.default_sandbox {
            "https://api.sandbox.push.apple.com"
        } else {
            "https://api.push.apple.com"
        };

        let url = format!("{}/3/device/{}", host, device_token);

        let payload = serde_json::json!({
            "aps": {
                "alert": {
                    "title": title,
                    "body": body,
                },
                "sound": "default",
            }
        });

        let mut request = self
            .http_client
            .post(&url)
            .header("authorization", format!("bearer {}", token))
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
                match status {
                    200 => ApnsPushResult::Success,
                    410 => ApnsPushResult::Gone,
                    403 => ApnsPushResult::AuthError,
                    429 | 503 => ApnsPushResult::Retry,
                    _ => {
                        let body_text = response.text().await.unwrap_or_default();
                        ApnsPushResult::OtherError(format!("HTTP {}: {}", status, body_text))
                    }
                }
            }
            Err(e) => ApnsPushResult::OtherError(format!("Request failed: {}", e)),
        }
    }
}
