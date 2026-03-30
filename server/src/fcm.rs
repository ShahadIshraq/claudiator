use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum FcmPushResult {
    Success,
    Unregistered,
    Retry,
    AuthError,
    OtherError(String),
}

#[derive(Serialize)]
struct GoogleJwtClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: u64,
    exp: u64,
}

struct CachedOAuthToken {
    access_token: String,
    expires_at: u64,
}

pub struct FcmClient {
    project_id: String,
    service_account_email: String,
    signing_key: EncodingKey,
    http_client: reqwest::Client,
    cached_token: RwLock<Option<CachedOAuthToken>>,
}

impl FcmClient {
    #[allow(dead_code)]
    pub(crate) fn new(
        service_account_path: &str,
        project_id: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let sa_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(service_account_path)?)?;
        let email = sa_json["client_email"]
            .as_str()
            .ok_or("Missing client_email in service account")?
            .to_string();
        let private_key_pem = sa_json["private_key"]
            .as_str()
            .ok_or("Missing private_key in service account")?;
        let signing_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
        let http_client = reqwest::Client::new();
        Ok(Self {
            project_id,
            service_account_email: email,
            signing_key,
            http_client,
            cached_token: RwLock::new(None),
        })
    }

    async fn get_or_refresh_token(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("System time error: {e}"))?
            .as_secs();

        {
            let cached = self.cached_token.read().await;
            if let Some(ref ct) = *cached {
                if now < ct.expires_at {
                    return Ok(ct.access_token.clone());
                }
            }
        }

        let claims = GoogleJwtClaims {
            iss: self.service_account_email.clone(),
            scope: "https://www.googleapis.com/auth/firebase.messaging".to_string(),
            aud: "https://oauth2.googleapis.com/token".to_string(),
            iat: now,
            exp: now + 3600,
        };
        let header = Header::new(Algorithm::RS256);
        let jwt = encode(&header, &claims, &self.signing_key)?;

        let resp = self
            .http_client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", jwt.as_str()),
            ])
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;
        let access_token = body["access_token"]
            .as_str()
            .ok_or("No access_token in OAuth response")?
            .to_string();

        {
            let mut cached = self.cached_token.write().await;
            *cached = Some(CachedOAuthToken {
                access_token: access_token.clone(),
                expires_at: now + 3300,
            });
        }

        Ok(access_token)
    }

    pub(crate) async fn send_push(
        &self,
        fcm_token: &str,
        title: &str,
        body: &str,
        notification_id: &str,
        session_id: &str,
        device_id: &str,
    ) -> FcmPushResult {
        let access_token = match self.get_or_refresh_token().await {
            Ok(t) => t,
            Err(e) => return FcmPushResult::OtherError(format!("OAuth token error: {e}")),
        };

        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.project_id
        );

        let payload = serde_json::json!({
            "message": {
                "token": fcm_token,
                "data": {
                    "title": title,
                    "body": body,
                    "notification_id": notification_id,
                    "session_id": session_id,
                    "device_id": device_id,
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
            Ok(response) => Self::status_to_push_result(response).await,
            Err(e) => FcmPushResult::OtherError(format!("Request failed: {e}")),
        }
    }

    async fn status_to_push_result(response: reqwest::Response) -> FcmPushResult {
        let status = response.status().as_u16();
        if matches!(status, 200 | 401 | 403 | 404 | 429 | 500 | 503) {
            map_status_to_push_result(status, "")
        } else {
            let body = response.text().await.unwrap_or_default();
            map_status_to_push_result(status, &body)
        }
    }
}

fn map_status_to_push_result(status: u16, body: &str) -> FcmPushResult {
    match status {
        200 => FcmPushResult::Success,
        401 | 403 => FcmPushResult::AuthError,
        404 => FcmPushResult::Unregistered,
        429 | 500 | 503 => FcmPushResult::Retry,
        _ => {
            if body.contains("UNREGISTERED") {
                FcmPushResult::Unregistered
            } else {
                FcmPushResult::OtherError(format!("HTTP {status}: {body}"))
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn status_200_maps_to_success() {
        assert!(matches!(
            map_status_to_push_result(200, ""),
            FcmPushResult::Success
        ));
    }

    #[test]
    fn status_404_maps_to_unregistered() {
        assert!(matches!(
            map_status_to_push_result(404, ""),
            FcmPushResult::Unregistered
        ));
    }

    #[test]
    fn status_401_maps_to_auth_error() {
        assert!(matches!(
            map_status_to_push_result(401, ""),
            FcmPushResult::AuthError
        ));
    }

    #[test]
    fn status_403_maps_to_auth_error() {
        assert!(matches!(
            map_status_to_push_result(403, ""),
            FcmPushResult::AuthError
        ));
    }

    #[test]
    fn status_429_maps_to_retry() {
        assert!(matches!(
            map_status_to_push_result(429, ""),
            FcmPushResult::Retry
        ));
    }

    #[test]
    fn status_500_maps_to_retry() {
        assert!(matches!(
            map_status_to_push_result(500, ""),
            FcmPushResult::Retry
        ));
    }

    #[test]
    fn status_503_maps_to_retry() {
        assert!(matches!(
            map_status_to_push_result(503, ""),
            FcmPushResult::Retry
        ));
    }

    #[test]
    fn unregistered_body_maps_to_unregistered() {
        assert!(matches!(
            map_status_to_push_result(400, r#"{"error":{"status":"UNREGISTERED"}}"#),
            FcmPushResult::Unregistered
        ));
    }
}
