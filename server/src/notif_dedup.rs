use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Cooldown window for low-priority notification types.
pub const NOTIF_COOLDOWN_WINDOW: Duration = Duration::from_secs(30);

/// Notification types that always fire immediately, bypassing the cooldown.
const HIGH_PRIORITY_TYPES: &[&str] = &["permission_prompt"];

/// Per-session, per-type cooldown state.
///
/// Key: `(session_id, notification_type)`, Value: `Instant` of last notification sent.
/// Each `(session, type)` pair has its own independent cooldown bucket.
pub type NotifCooldownMap = Mutex<HashMap<(String, String), Instant>>;

/// Returns `true` if the notification should be sent, `false` if it should be suppressed.
///
/// - **High-priority** types (`permission_prompt`) always return `true`.
/// - **Low-priority** types (`stop`, `idle_prompt`) return `true` only when no notification
///   of the same type was sent for this session within [`NOTIF_COOLDOWN_WINDOW`].
///
/// On a `true` return, the map entry is updated to the current time.
pub fn should_send_notification(
    map: &NotifCooldownMap,
    session_id: &str,
    notif_type: &str,
) -> bool {
    if HIGH_PRIORITY_TYPES.contains(&notif_type) {
        return true;
    }

    let mut guard = map
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let now = Instant::now();

    // Evict expired entries to prevent unbounded memory growth.
    guard.retain(|_, last_sent| now.duration_since(*last_sent) < NOTIF_COOLDOWN_WINDOW);

    let key = (session_id.to_string(), notif_type.to_string());

    if guard
        .get(&key)
        .is_some_and(|last| now.duration_since(*last) < NOTIF_COOLDOWN_WINDOW)
    {
        return false;
    }

    guard.insert(key, now);
    true
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::unchecked_duration_subtraction)]
mod tests {
    use super::*;

    fn make_map() -> NotifCooldownMap {
        Mutex::new(HashMap::new())
    }

    #[test]
    fn test_permission_prompt_always_fires() {
        let map = make_map();
        // High-priority: never suppressed regardless of call count.
        assert!(should_send_notification(
            &map,
            "sess-1",
            "permission_prompt"
        ));
        assert!(should_send_notification(
            &map,
            "sess-1",
            "permission_prompt"
        ));
        assert!(should_send_notification(
            &map,
            "sess-1",
            "permission_prompt"
        ));
    }

    #[test]
    fn test_stop_fires_first_time() {
        let map = make_map();
        assert!(should_send_notification(&map, "sess-1", "stop"));
    }

    #[test]
    fn test_stop_suppressed_within_window() {
        let map = make_map();
        assert!(should_send_notification(&map, "sess-1", "stop")); // first: fires
        assert!(!should_send_notification(&map, "sess-1", "stop")); // second: suppressed
        assert!(!should_send_notification(&map, "sess-1", "stop")); // third: suppressed
    }

    #[test]
    fn test_idle_prompt_suppressed_within_window() {
        let map = make_map();
        assert!(should_send_notification(&map, "sess-1", "idle_prompt"));
        assert!(!should_send_notification(&map, "sess-1", "idle_prompt"));
    }

    #[test]
    fn test_different_sessions_have_independent_cooldowns() {
        let map = make_map();
        assert!(should_send_notification(&map, "sess-1", "stop")); // sess-1 fires
        assert!(should_send_notification(&map, "sess-2", "stop")); // sess-2 fires independently
        assert!(!should_send_notification(&map, "sess-1", "stop")); // sess-1 suppressed
        assert!(!should_send_notification(&map, "sess-2", "stop")); // sess-2 suppressed
    }

    #[test]
    fn test_different_types_have_independent_cooldowns_per_session() {
        let map = make_map();
        assert!(should_send_notification(&map, "sess-1", "stop")); // stop fires
        assert!(should_send_notification(&map, "sess-1", "idle_prompt")); // idle_prompt fires independently
        assert!(!should_send_notification(&map, "sess-1", "stop")); // stop suppressed
        assert!(!should_send_notification(&map, "sess-1", "idle_prompt")); // idle_prompt suppressed
    }

    #[test]
    fn test_stop_suppressed_but_permission_prompt_still_fires() {
        let map = make_map();
        // stop fires then gets suppressed
        assert!(should_send_notification(&map, "sess-1", "stop"));
        assert!(!should_send_notification(&map, "sess-1", "stop"));
        // permission_prompt always fires regardless
        assert!(should_send_notification(
            &map,
            "sess-1",
            "permission_prompt"
        ));
        assert!(should_send_notification(
            &map,
            "sess-1",
            "permission_prompt"
        ));
    }

    #[test]
    fn test_fires_again_after_window_expires() {
        let map = make_map();
        assert!(should_send_notification(&map, "sess-1", "stop")); // fires

        // Backdate the entry to simulate window expiry.
        {
            let mut guard = map.lock().unwrap();
            let key = ("sess-1".to_string(), "stop".to_string());
            if let Some(entry) = guard.get_mut(&key) {
                *entry = Instant::now() - NOTIF_COOLDOWN_WINDOW - Duration::from_millis(1);
            }
        }

        assert!(should_send_notification(&map, "sess-1", "stop")); // fires again after window
    }

    #[test]
    fn test_eviction_removes_expired_entries() {
        let map = make_map();
        // Seed an entry for sess-1/stop.
        assert!(should_send_notification(&map, "sess-1", "stop"));
        let initial_len = map.lock().unwrap().len();
        assert_eq!(initial_len, 1);

        // Backdate it past the window.
        {
            let mut guard = map.lock().unwrap();
            let key = ("sess-1".to_string(), "stop".to_string());
            if let Some(entry) = guard.get_mut(&key) {
                *entry = Instant::now() - NOTIF_COOLDOWN_WINDOW - Duration::from_millis(1);
            }
        }

        // A new call triggers eviction of expired entries.
        assert!(should_send_notification(&map, "sess-2", "stop"));

        // sess-1 entry must have been evicted.
        assert!(!map
            .lock()
            .unwrap()
            .contains_key(&("sess-1".to_string(), "stop".to_string())));
    }

    #[test]
    fn test_unknown_type_treated_as_low_priority() {
        // Any type not in HIGH_PRIORITY_TYPES is subject to the cooldown.
        let map = make_map();
        assert!(should_send_notification(&map, "sess-1", "future_type"));
        assert!(!should_send_notification(&map, "sess-1", "future_type"));
    }

    #[test]
    fn test_map_is_empty_initially() {
        let map = make_map();
        assert!(map.lock().unwrap().is_empty());
    }

    #[test]
    fn test_map_grows_after_first_low_priority_send() {
        let map = make_map();
        should_send_notification(&map, "sess-1", "stop");
        assert_eq!(map.lock().unwrap().len(), 1);
        should_send_notification(&map, "sess-1", "idle_prompt");
        assert_eq!(map.lock().unwrap().len(), 2);
        should_send_notification(&map, "sess-2", "stop");
        assert_eq!(map.lock().unwrap().len(), 3);
    }

    #[test]
    fn test_suppressed_call_does_not_update_timestamp() {
        let map = make_map();
        assert!(should_send_notification(&map, "sess-1", "stop")); // fires, records t0

        let t0 = {
            let guard = map.lock().unwrap();
            *guard
                .get(&("sess-1".to_string(), "stop".to_string()))
                .unwrap()
        };

        // Suppressed call — timestamp should stay at t0.
        assert!(!should_send_notification(&map, "sess-1", "stop"));

        let t1 = {
            let guard = map.lock().unwrap();
            *guard
                .get(&("sess-1".to_string(), "stop".to_string()))
                .unwrap()
        };

        assert_eq!(t0, t1);
    }
}
