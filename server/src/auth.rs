use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::HeaderMap;
use chrono::{SecondsFormat, Utc};

use crate::db::queries;
use crate::error::AppError;
use crate::router::AppState;

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

    IpAddr::from([0u8, 0, 0, 0])
}

/// Returns `Err(AppError::RateLimited)` if the IP has exceeded `MAX_FAILURES`
/// within `FAILURE_WINDOW`.
pub fn check_rate_limit(map: &AuthFailureMap, ip: IpAddr) -> Result<(), AppError> {
    let mut guard = map
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

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

    if now.duration_since(entry.1) >= FAILURE_WINDOW {
        *entry = (0, now);
    }

    entry.0 = entry.0.saturating_add(1);
}

// ── Scope & AuthenticatedKey ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Read,
    Write,
}

impl Scope {
    fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            _ => None,
        }
    }
}

pub fn parse_scopes(s: &str) -> Vec<Scope> {
    s.split(',').filter_map(Scope::from_str).collect()
}

pub fn format_scopes(scopes: &[Scope]) -> String {
    scopes
        .iter()
        .map(|s| match s {
            Scope::Read => "read",
            Scope::Write => "write",
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub struct AuthenticatedKey {
    pub id: Option<String>,
    pub name: String,
    pub scopes: Vec<Scope>,
}

// ── Typed extractors ──────────────────────────────────────────────────────────

/// Extractor that requires a valid key with `read` scope.
pub struct ReadAuth(pub AuthenticatedKey);

/// Extractor that requires a valid key with `write` scope.
pub struct WriteAuth(pub AuthenticatedKey);

/// Extractor for admin endpoints: requires localhost origin + master key.
pub struct AdminAuth(pub AuthenticatedKey);

// ── Core resolution logic ─────────────────────────────────────────────────────

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

/// Resolves and validates the bearer token, checking the required scope.
/// Updates `last_used` for DB keys on successful auth.
fn resolve_auth(
    headers: &HeaderMap,
    state: &Arc<AppState>,
    required_scope: Scope,
) -> Result<AuthenticatedKey, AppError> {
    let ip = extract_client_ip(headers);
    check_rate_limit(&state.auth_failures, ip)?;

    let token = match extract_bearer_token(headers) {
        Some(t) => t,
        None => {
            record_auth_failure(&state.auth_failures, ip);
            return Err(AppError::Unauthorized);
        }
    };

    // Master key — always read+write
    if token == state.master_key {
        return Ok(AuthenticatedKey {
            id: None,
            name: "master".to_string(),
            scopes: vec![Scope::Read, Scope::Write],
        });
    }

    // DB key lookup
    let conn = state
        .db_pool
        .get()
        .map_err(|e| AppError::Internal(format!("DB pool error: {e}")))?;

    match queries::find_api_key_by_key(&conn, token)? {
        Some(row) => {
            let scopes = parse_scopes(&row.scopes);

            if !scopes.contains(&required_scope) {
                return Err(AppError::Forbidden);
            }

            let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let _ = queries::update_api_key_last_used(&conn, &row.id, &now);

            Ok(AuthenticatedKey {
                id: Some(row.id),
                name: row.name,
                scopes,
            })
        }
        None => {
            record_auth_failure(&state.auth_failures, ip);
            Err(AppError::Unauthorized)
        }
    }
}

// ── FromRequestParts implementations ─────────────────────────────────────────

impl FromRequestParts<Arc<AppState>> for ReadAuth {
    type Rejection = AppError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 Arc<AppState>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, AppError>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { resolve_auth(&parts.headers, state, Scope::Read).map(ReadAuth) })
    }
}

impl FromRequestParts<Arc<AppState>> for WriteAuth {
    type Rejection = AppError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 Arc<AppState>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, AppError>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { resolve_auth(&parts.headers, state, Scope::Write).map(WriteAuth) })
    }
}

impl FromRequestParts<Arc<AppState>> for AdminAuth {
    type Rejection = AppError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 Arc<AppState>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, AppError>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            use axum::extract::ConnectInfo;
            use std::net::SocketAddr;

            // Require localhost origin
            let addr = parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ci| ci.0)
                .ok_or(AppError::Forbidden)?;

            if !addr.ip().is_loopback() {
                return Err(AppError::Forbidden);
            }

            // Require master key
            let ip = extract_client_ip(&parts.headers);
            check_rate_limit(&state.auth_failures, ip)?;

            let token = match extract_bearer_token(&parts.headers) {
                Some(t) => t,
                None => {
                    record_auth_failure(&state.auth_failures, ip);
                    return Err(AppError::Unauthorized);
                }
            };

            if token != state.master_key {
                record_auth_failure(&state.auth_failures, ip);
                return Err(AppError::Unauthorized);
            }

            Ok(AdminAuth(AuthenticatedKey {
                id: None,
                name: "master".to_string(),
                scopes: vec![Scope::Read, Scope::Write],
            }))
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn make_map() -> AuthFailureMap {
        Mutex::new(HashMap::new())
    }

    fn test_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))
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

    #[test]
    fn test_parse_scopes_read() {
        let scopes = parse_scopes("read");
        assert_eq!(scopes, vec![Scope::Read]);
    }

    #[test]
    fn test_parse_scopes_write() {
        let scopes = parse_scopes("write");
        assert_eq!(scopes, vec![Scope::Write]);
    }

    #[test]
    fn test_parse_scopes_both() {
        let scopes = parse_scopes("read,write");
        assert_eq!(scopes, vec![Scope::Read, Scope::Write]);
    }

    #[test]
    fn test_format_scopes() {
        let scopes = vec![Scope::Read, Scope::Write];
        assert_eq!(format_scopes(&scopes), "read,write");
    }

    #[test]
    fn test_parse_scopes_invalid_values_skipped() {
        let scopes = parse_scopes("read,foo,write");
        assert_eq!(scopes, vec![Scope::Read, Scope::Write]);
    }

    #[test]
    fn test_parse_scopes_empty_string() {
        let scopes = parse_scopes("");
        assert!(scopes.is_empty());
    }

    #[test]
    fn test_parse_scopes_with_whitespace() {
        let scopes = parse_scopes(" read , write ");
        assert_eq!(scopes, vec![Scope::Read, Scope::Write]);
    }

    #[test]
    fn test_format_scopes_read_only() {
        assert_eq!(format_scopes(&[Scope::Read]), "read");
    }

    #[test]
    fn test_format_scopes_write_only() {
        assert_eq!(format_scopes(&[Scope::Write]), "write");
    }

    #[test]
    fn test_format_scopes_empty() {
        assert_eq!(format_scopes(&[]), "");
    }

    #[test]
    fn test_extract_bearer_token_valid() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer abc123".parse().unwrap());
        assert_eq!(extract_bearer_token(&headers), Some("abc123"));
    }

    #[test]
    fn test_extract_bearer_token_missing_header() {
        let headers = HeaderMap::new();
        assert_eq!(extract_bearer_token(&headers), None);
    }

    #[test]
    fn test_extract_bearer_token_wrong_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic abc123".parse().unwrap());
        assert_eq!(extract_bearer_token(&headers), None);
    }

    #[test]
    fn test_extract_bearer_token_empty_after_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer ".parse().unwrap());
        assert_eq!(extract_bearer_token(&headers), Some(""));
    }

    #[test]
    fn test_extract_bearer_token_no_space() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearerabc123".parse().unwrap());
        assert_eq!(extract_bearer_token(&headers), None);
    }

    #[test]
    fn test_scope_equality() {
        assert_eq!(Scope::Read, Scope::Read);
        assert_eq!(Scope::Write, Scope::Write);
        assert_ne!(Scope::Read, Scope::Write);
    }
}
