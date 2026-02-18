use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::http::HeaderMap;

use crate::error::AppError;

/// Maximum number of failed auth attempts before rate-limiting an IP.
const MAX_FAILURES: u32 = 10;

/// Time window within which failures are counted. After this window the
/// counter resets automatically.
const FAILURE_WINDOW: Duration = Duration::from_secs(5 * 60);

/// Per-IP state: (`failure_count`, `window_start`).
pub type AuthFailureMap = Mutex<HashMap<IpAddr, (u32, Instant)>>;

/// Extracts the client IP from request headers.
///
/// Checks `X-Forwarded-For` first (first address in the list), then
/// `X-Real-IP`. Falls back to `0.0.0.0` when neither header is present so
/// that unknown clients share a single rate-limit bucket rather than being
/// exempt from limiting.
pub fn extract_client_ip(headers: &HeaderMap) -> IpAddr {
    // X-Forwarded-For may contain a comma-separated list; take the first.
    if let Some(forwarded) = headers.get("X-Forwarded-For").and_then(|v| v.to_str().ok()) {
        let first = forwarded.split(',').next().unwrap_or("").trim();
        if let Ok(ip) = first.parse::<IpAddr>() {
            return ip;
        }
    }

    if let Some(real_ip) = headers.get("X-Real-IP").and_then(|v| v.to_str().ok()) {
        if let Ok(ip) = real_ip.trim().parse::<IpAddr>() {
            return ip;
        }
    }

    // Sentinel: all unknown-origin requests share one bucket.
    IpAddr::from([0u8, 0, 0, 0])
}

/// Returns `Err(AppError::RateLimited)` if the IP has exceeded `MAX_FAILURES`
/// within `FAILURE_WINDOW`. Cleans up expired entries for other IPs while
/// holding the lock.
pub fn check_rate_limit(map: &AuthFailureMap, ip: IpAddr) -> Result<(), AppError> {
    let mut guard = map
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    // Opportunistic sweep: remove entries whose window has expired.
    let now = Instant::now();
    guard.retain(|_, (_, start)| now.duration_since(*start) < FAILURE_WINDOW);

    let is_limited = guard
        .get(&ip)
        .is_some_and(|(count, _)| *count >= MAX_FAILURES);
    drop(guard);

    if is_limited {
        return Err(AppError::RateLimited);
    }

    Ok(())
}

/// Records a failed authentication attempt for `ip`.
#[allow(clippy::significant_drop_tightening)]
pub fn record_auth_failure(map: &AuthFailureMap, ip: IpAddr) {
    let mut guard = map
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let now = Instant::now();

    let entry = guard.entry(ip).or_insert((0, now));

    // If the existing window has expired, start a fresh one.
    if now.duration_since(entry.1) >= FAILURE_WINDOW {
        *entry = (0, now);
    }

    entry.0 = entry.0.saturating_add(1);
}

pub fn check_auth(headers: &HeaderMap, api_key: &str) -> Result<(), AppError> {
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use std::net::Ipv4Addr;

    fn make_map() -> AuthFailureMap {
        Mutex::new(HashMap::new())
    }

    fn test_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))
    }

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

    #[test]
    fn test_rate_limit_allows_under_threshold() {
        let map = make_map();
        let ip = test_ip();

        for _ in 0..MAX_FAILURES - 1 {
            record_auth_failure(&map, ip);
        }

        assert!(check_rate_limit(&map, ip).is_ok());
    }

    #[test]
    fn test_rate_limit_blocks_at_threshold() {
        let map = make_map();
        let ip = test_ip();

        for _ in 0..MAX_FAILURES {
            record_auth_failure(&map, ip);
        }

        assert!(matches!(
            check_rate_limit(&map, ip),
            Err(AppError::RateLimited)
        ));
    }

    #[test]
    fn test_rate_limit_different_ips_independent() {
        let map = make_map();
        let ip_a = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let ip_b = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));

        for _ in 0..MAX_FAILURES {
            record_auth_failure(&map, ip_a);
        }

        // ip_b should not be affected.
        assert!(check_rate_limit(&map, ip_b).is_ok());
    }

    #[test]
    fn test_extract_client_ip_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Forwarded-For", "203.0.113.5, 10.0.0.1".parse().unwrap());
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, "203.0.113.5".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_extract_client_ip_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", "198.51.100.7".parse().unwrap());
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, "198.51.100.7".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_extract_client_ip_fallback() {
        let headers = HeaderMap::new();
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, IpAddr::from([0u8, 0, 0, 0]));
    }
}
