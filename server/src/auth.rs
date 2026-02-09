use axum::http::HeaderMap;

use crate::error::AppError;

pub(crate) fn check_auth(headers: &HeaderMap, api_key: &str) -> Result<(), AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let expected = format!("Bearer {api_key}");
    if auth_header == expected {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_check_auth_valid() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer test-key".parse().unwrap());
        assert!(check_auth(&headers, "test-key").is_ok());
    }

    #[test]
    fn test_check_auth_invalid() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer wrong-key".parse().unwrap());
        assert!(matches!(
            check_auth(&headers, "test-key"),
            Err(AppError::Unauthorized)
        ));
    }

    #[test]
    fn test_check_auth_missing() {
        let headers = HeaderMap::new();
        assert!(matches!(
            check_auth(&headers, "test-key"),
            Err(AppError::Unauthorized)
        ));
    }

    #[test]
    fn test_check_auth_malformed() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "InvalidFormat".parse().unwrap());
        assert!(matches!(
            check_auth(&headers, "test-key"),
            Err(AppError::Unauthorized)
        ));
    }
}
