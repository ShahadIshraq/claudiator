use axum::http::HeaderMap;

use crate::error::AppError;

pub fn check_auth(headers: &HeaderMap, api_key: &str) -> Result<(), AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let expected = format!("Bearer {}", api_key);
    if auth_header == expected {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}
