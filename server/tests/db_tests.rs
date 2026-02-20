#![allow(clippy::unwrap_used)]
#![allow(unused_variables)]
#![allow(missing_docs)]

use claudiator_server::db::{migrations, pool, queries};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

type DbPool = Pool<SqliteConnectionManager>;

fn test_pool() -> DbPool {
    let pool = pool::create_pool(":memory:").unwrap();
    // For :memory: databases with r2d2, use max_size(1) since each connection
    // creates a separate database
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::builder().max_size(1).build(manager).unwrap();
    migrations::run(&pool).unwrap();
    pool
}

#[test]
fn test_migration_idempotency() {
    let pool = pool::create_pool(":memory:").unwrap();
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::builder().max_size(1).build(manager).unwrap();

    // Run migrations twice
    migrations::run(&pool).unwrap();
    migrations::run(&pool).unwrap();

    // Should not panic or error
    let conn = pool.get().unwrap();
    let result: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(result > 0);
}

#[test]
fn test_upsert_device() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Insert
    queries::upsert_device(&conn, "device-1", "My Device", "macos", &now).unwrap();

    // Verify insert
    let devices = queries::list_devices(&conn).unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].device_id, "device-1");
    assert_eq!(devices[0].device_name, "My Device");
    assert_eq!(devices[0].platform, "macos");

    // Update
    let later = chrono::Utc::now().to_rfc3339();
    queries::upsert_device(&conn, "device-1", "Updated Device", "macos", &later).unwrap();

    // Verify update
    let devices = queries::list_devices(&conn).unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].device_name, "Updated Device");
}

#[test]
fn test_upsert_session() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Setup device
    queries::upsert_device(&conn, "device-1", "My Device", "macos", &now).unwrap();

    // Insert session
    queries::upsert_session(
        &conn,
        "session-1",
        "device-1",
        &now,
        Some("active"),
        Some("/home/user"),
        Some("Initial Title"),
    )
    .unwrap();

    // Verify insert
    let sessions = queries::list_sessions(&conn, "device-1", None, 10).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, "session-1");
    assert_eq!(sessions[0].status, "active");
    assert_eq!(sessions[0].title, Some("Initial Title".to_string()));

    // Update without title (should preserve existing title)
    queries::upsert_session(
        &conn,
        "session-1",
        "device-1",
        &now,
        Some("ended"),
        None,
        None,
    )
    .unwrap();

    // Verify title was not overwritten
    let sessions = queries::list_sessions(&conn, "device-1", None, 10).unwrap();
    assert_eq!(sessions[0].status, "ended");
    assert_eq!(sessions[0].title, Some("Initial Title".to_string()));
}

#[test]
fn test_insert_and_list_events() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Setup
    queries::upsert_device(&conn, "device-1", "My Device", "macos", &now).unwrap();
    queries::upsert_session(&conn, "session-1", "device-1", &now, None, None, None).unwrap();

    // Insert event
    let event_id = queries::insert_event(
        &conn,
        "device-1",
        "session-1",
        "tool-use",
        &now,
        &now,
        Some("bash"),
        Some("info"),
        r#"{"message":"test"}"#,
    )
    .unwrap();

    assert!(event_id > 0);

    // List events
    let events = queries::list_events(&conn, "session-1", 10).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].hook_event_name, "tool-use");
    assert_eq!(events[0].tool_name, Some("bash".to_string()));
    assert_eq!(events[0].notification_type, Some("info".to_string()));
}

#[test]
fn test_list_devices_with_active_sessions() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Setup devices
    queries::upsert_device(&conn, "device-1", "Device 1", "macos", &now).unwrap();
    queries::upsert_device(&conn, "device-2", "Device 2", "linux", &now).unwrap();

    // Device 1: 2 active, 1 ended
    queries::upsert_session(&conn, "s1", "device-1", &now, Some("active"), None, None).unwrap();
    queries::upsert_session(&conn, "s2", "device-1", &now, Some("active"), None, None).unwrap();
    queries::upsert_session(&conn, "s3", "device-1", &now, Some("ended"), None, None).unwrap();

    // Device 2: 1 active
    queries::upsert_session(&conn, "s4", "device-2", &now, Some("active"), None, None).unwrap();

    let devices = queries::list_devices(&conn).unwrap();
    assert_eq!(devices.len(), 2);

    let dev1 = devices.iter().find(|d| d.device_id == "device-1").unwrap();
    assert_eq!(dev1.active_sessions, 2);

    let dev2 = devices.iter().find(|d| d.device_id == "device-2").unwrap();
    assert_eq!(dev2.active_sessions, 1);
}

#[test]
fn test_list_sessions_with_status_filter() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    queries::upsert_device(&conn, "device-1", "Device 1", "macos", &now).unwrap();
    queries::upsert_session(&conn, "s1", "device-1", &now, Some("active"), None, None).unwrap();
    queries::upsert_session(&conn, "s2", "device-1", &now, Some("ended"), None, None).unwrap();
    queries::upsert_session(&conn, "s3", "device-1", &now, Some("active"), None, None).unwrap();

    // Filter by active
    let active = queries::list_sessions(&conn, "device-1", Some("active"), 10).unwrap();
    assert_eq!(active.len(), 2);

    // Filter by ended
    let ended = queries::list_sessions(&conn, "device-1", Some("ended"), 10).unwrap();
    assert_eq!(ended.len(), 1);

    // No filter
    let all = queries::list_sessions(&conn, "device-1", None, 10).unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn test_list_sessions_with_limit() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    queries::upsert_device(&conn, "device-1", "Device 1", "macos", &now).unwrap();
    for i in 1..=5 {
        queries::upsert_session(
            &conn,
            &format!("s{i}"),
            "device-1",
            &now,
            Some("active"),
            None,
            None,
        )
        .unwrap();
    }

    let sessions = queries::list_sessions(&conn, "device-1", None, 3).unwrap();
    assert_eq!(sessions.len(), 3);
}

#[test]
fn test_list_all_sessions() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    queries::upsert_device(&conn, "device-1", "Device 1", "macos", &now).unwrap();
    queries::upsert_device(&conn, "device-2", "Device 2", "linux", &now).unwrap();

    queries::upsert_session(&conn, "s1", "device-1", &now, Some("active"), None, None).unwrap();
    queries::upsert_session(&conn, "s2", "device-2", &now, Some("active"), None, None).unwrap();

    let sessions = queries::list_all_sessions(&conn, None, 10).unwrap();
    assert_eq!(sessions.len(), 2);

    let sessions_filtered = queries::list_all_sessions(&conn, Some("active"), 10).unwrap();
    assert_eq!(sessions_filtered.len(), 2);
}

#[test]
fn test_push_token_lifecycle() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Insert
    queries::upsert_push_token(&conn, "ios", "token-123", &now, false).unwrap();

    // List
    let tokens = queries::list_push_tokens(&conn).unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].platform, "ios");
    assert_eq!(tokens[0].push_token, "token-123");
    assert!(!tokens[0].sandbox);

    // Update
    queries::upsert_push_token(&conn, "ios", "token-123", &now, true).unwrap();
    let tokens = queries::list_push_tokens(&conn).unwrap();
    assert_eq!(tokens.len(), 1);
    assert!(tokens[0].sandbox);

    // Delete
    queries::delete_push_token(&conn, "token-123").unwrap();
    let tokens = queries::list_push_tokens(&conn).unwrap();
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_notification_lifecycle() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Setup
    queries::upsert_device(&conn, "device-1", "Device", "macos", &now).unwrap();
    queries::upsert_session(&conn, "session-1", "device-1", &now, None, None, None).unwrap();
    let event_id = queries::insert_event(
        &conn,
        "device-1",
        "session-1",
        "tool-use",
        &now,
        &now,
        None,
        None,
        "{}",
    )
    .unwrap();

    // Insert notification
    queries::insert_notification(
        &conn,
        "notif-1",
        event_id,
        "session-1",
        "device-1",
        "Test Title",
        "Test Body",
        "info",
        Some(r#"{"key":"value"}"#),
        &now,
    )
    .unwrap();

    // List all
    let notifs = queries::list_notifications(&conn, None, 10).unwrap();
    assert_eq!(notifs.len(), 1);
    assert_eq!(notifs[0].title, "Test Title");

    // Insert another (with later timestamp)
    let later = (chrono::Utc::now() + chrono::Duration::seconds(1)).to_rfc3339();
    queries::insert_notification(
        &conn,
        "notif-2",
        event_id,
        "session-1",
        "device-1",
        "Second",
        "Body",
        "warning",
        None,
        &later,
    )
    .unwrap();

    // List notifications after the first notification's timestamp
    let notifs = queries::list_notifications(&conn, Some(&now), 10).unwrap();
    assert_eq!(notifs.len(), 1);
    assert_eq!(notifs[0].id, "notif-2");
}

#[test]
fn test_delete_expired_notifications() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    // Setup
    let now = chrono::Utc::now().to_rfc3339();
    queries::upsert_device(&conn, "device-1", "Device", "macos", &now).unwrap();
    queries::upsert_session(&conn, "session-1", "device-1", &now, None, None, None).unwrap();
    let event_id = queries::insert_event(
        &conn,
        "device-1",
        "session-1",
        "tool-use",
        &now,
        &now,
        None,
        None,
        "{}",
    )
    .unwrap();

    // Insert old notification (25 hours ago)
    let old_time = chrono::Utc::now() - chrono::Duration::hours(25);
    let old_time_str = old_time.to_rfc3339();
    queries::insert_notification(
        &conn,
        "old-notif",
        event_id,
        "session-1",
        "device-1",
        "Old",
        "Body",
        "info",
        None,
        &old_time_str,
    )
    .unwrap();

    // Insert recent notification
    queries::insert_notification(
        &conn,
        "new-notif",
        event_id,
        "session-1",
        "device-1",
        "New",
        "Body",
        "info",
        None,
        &now,
    )
    .unwrap();

    // Delete expired
    let deleted = queries::delete_expired_notifications(&conn).unwrap();
    assert_eq!(deleted, 1);

    // Verify only recent remains
    let notifs = queries::list_notifications(&conn, None, 10).unwrap();
    assert_eq!(notifs.len(), 1);
    assert_eq!(notifs[0].id, "new-notif");
}

#[test]
fn test_metadata_operations() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    // Get non-existent key
    let value = queries::get_metadata(&conn, "test-key").unwrap();
    assert!(value.is_none());

    // Set a value
    queries::set_metadata(&conn, "test-key", "test-value").unwrap();

    // Get the value
    let value = queries::get_metadata(&conn, "test-key").unwrap();
    assert_eq!(value, Some("test-value".to_string()));

    // Update the value
    queries::set_metadata(&conn, "test-key", "updated-value").unwrap();

    // Verify update
    let value = queries::get_metadata(&conn, "test-key").unwrap();
    assert_eq!(value, Some("updated-value".to_string()));
}

#[test]
fn test_metadata_multiple_keys() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    // Set multiple keys
    queries::set_metadata(&conn, "key1", "value1").unwrap();
    queries::set_metadata(&conn, "key2", "value2").unwrap();
    queries::set_metadata(&conn, "key3", "value3").unwrap();

    // Verify all keys
    assert_eq!(
        queries::get_metadata(&conn, "key1").unwrap(),
        Some("value1".to_string())
    );
    assert_eq!(
        queries::get_metadata(&conn, "key2").unwrap(),
        Some("value2".to_string())
    );
    assert_eq!(
        queries::get_metadata(&conn, "key3").unwrap(),
        Some("value3".to_string())
    );
}

#[test]
fn test_acknowledge_notifications_basic() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Setup
    queries::upsert_device(&conn, "device-1", "Device", "macos", &now).unwrap();
    queries::upsert_session(&conn, "session-1", "device-1", &now, None, None, None).unwrap();
    let event_id = queries::insert_event(
        &conn,
        "device-1",
        "session-1",
        "tool-use",
        &now,
        &now,
        None,
        None,
        "{}",
    )
    .unwrap();

    // Insert notifications
    queries::insert_notification(
        &conn,
        "notif-1",
        event_id,
        "session-1",
        "device-1",
        "Title 1",
        "Body",
        "info",
        None,
        &now,
    )
    .unwrap();

    queries::insert_notification(
        &conn,
        "notif-2",
        event_id,
        "session-1",
        "device-1",
        "Title 2",
        "Body",
        "info",
        None,
        &now,
    )
    .unwrap();

    // Acknowledge one notification
    queries::acknowledge_notifications(&conn, &["notif-1".to_string()]).unwrap();

    // Verify acknowledged status
    let notifs = queries::list_notifications(&conn, None, 10).unwrap();
    let notif1 = notifs.iter().find(|n| n.id == "notif-1").unwrap();
    let notif2 = notifs.iter().find(|n| n.id == "notif-2").unwrap();

    assert!(notif1.acknowledged);
    assert!(!notif2.acknowledged);
}

#[test]
fn test_acknowledge_notifications_multiple() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Setup
    queries::upsert_device(&conn, "device-1", "Device", "macos", &now).unwrap();
    queries::upsert_session(&conn, "session-1", "device-1", &now, None, None, None).unwrap();
    let event_id = queries::insert_event(
        &conn,
        "device-1",
        "session-1",
        "tool-use",
        &now,
        &now,
        None,
        None,
        "{}",
    )
    .unwrap();

    // Insert notifications
    for i in 1..=5 {
        queries::insert_notification(
            &conn,
            &format!("notif-{i}"),
            event_id,
            "session-1",
            "device-1",
            "Title",
            "Body",
            "info",
            None,
            &now,
        )
        .unwrap();
    }

    // Acknowledge multiple notifications
    queries::acknowledge_notifications(
        &conn,
        &[
            "notif-1".to_string(),
            "notif-2".to_string(),
            "notif-3".to_string(),
        ],
    )
    .unwrap();

    // Verify
    let notifs = queries::list_notifications(&conn, None, 10).unwrap();
    let acked_count = notifs.iter().filter(|n| n.acknowledged).count();
    assert_eq!(acked_count, 3);
}

#[test]
fn test_acknowledge_notifications_empty() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    // Should not error on empty array
    queries::acknowledge_notifications(&conn, &[]).unwrap();
}

#[test]
fn test_acknowledge_notifications_nonexistent() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    // Should not error on non-existent IDs
    queries::acknowledge_notifications(&conn, &["nonexistent".to_string()]).unwrap();
}

#[test]
fn test_list_notifications_with_after_timestamp() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Setup
    queries::upsert_device(&conn, "device-1", "Device", "macos", &now).unwrap();
    queries::upsert_session(&conn, "session-1", "device-1", &now, None, None, None).unwrap();
    let event_id = queries::insert_event(
        &conn,
        "device-1",
        "session-1",
        "tool-use",
        &now,
        &now,
        None,
        None,
        "{}",
    )
    .unwrap();

    // Insert notification with specific timestamp
    let timestamp1 = "2024-01-01T10:00:00Z";
    queries::insert_notification(
        &conn,
        "notif-1",
        event_id,
        "session-1",
        "device-1",
        "First",
        "Body",
        "info",
        None,
        timestamp1,
    )
    .unwrap();

    // Insert second notification with later timestamp
    let timestamp2 = "2024-01-01T11:00:00Z";
    queries::insert_notification(
        &conn,
        "notif-2",
        event_id,
        "session-1",
        "device-1",
        "Second",
        "Body",
        "info",
        None,
        timestamp2,
    )
    .unwrap();

    // List all notifications
    let all_notifs = queries::list_notifications(&conn, None, 10).unwrap();
    assert_eq!(all_notifs.len(), 2);

    // List notifications after first timestamp
    let notifs = queries::list_notifications(&conn, Some(timestamp1), 10).unwrap();
    assert_eq!(notifs.len(), 1);
    assert_eq!(notifs[0].id, "notif-2");
}

#[test]
fn test_timestamp_pagination_ordering() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Setup
    queries::upsert_device(&conn, "device-1", "Device", "macos", &now).unwrap();
    queries::upsert_session(&conn, "session-1", "device-1", &now, None, None, None).unwrap();
    let event_id = queries::insert_event(
        &conn,
        "device-1",
        "session-1",
        "tool-use",
        &now,
        &now,
        None,
        None,
        "{}",
    )
    .unwrap();

    // Insert notifications with ascending timestamps
    queries::insert_notification(
        &conn,
        "notif-1",
        event_id,
        "session-1",
        "device-1",
        "First",
        "Body",
        "info",
        None,
        "2024-01-01T10:00:00Z",
    )
    .unwrap();

    queries::insert_notification(
        &conn,
        "notif-2",
        event_id,
        "session-1",
        "device-1",
        "Second",
        "Body",
        "info",
        None,
        "2024-01-01T11:00:00Z",
    )
    .unwrap();

    queries::insert_notification(
        &conn,
        "notif-3",
        event_id,
        "session-1",
        "device-1",
        "Third",
        "Body",
        "info",
        None,
        "2024-01-01T12:00:00Z",
    )
    .unwrap();

    // List all - should be in ascending order by timestamp
    let notifs = queries::list_notifications(&conn, None, 10).unwrap();
    assert_eq!(notifs.len(), 3);
    assert_eq!(notifs[0].id, "notif-1");
    assert_eq!(notifs[1].id, "notif-2");
    assert_eq!(notifs[2].id, "notif-3");
}

#[test]
fn test_get_session_title_with_title() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    queries::upsert_device(&conn, "device-1", "My Device", "macos", &now).unwrap();
    queries::upsert_session(
        &conn,
        "session-1",
        "device-1",
        &now,
        Some("active"),
        None,
        Some("Fix login bug"),
    )
    .unwrap();

    let title = queries::get_session_title(&conn, "session-1").unwrap();
    assert_eq!(title, Some("Fix login bug".to_string()));
}

#[test]
fn test_get_session_title_without_title() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    queries::upsert_device(&conn, "device-1", "My Device", "macos", &now).unwrap();
    queries::upsert_session(
        &conn,
        "session-1",
        "device-1",
        &now,
        Some("active"),
        None,
        None,
    )
    .unwrap();

    let title = queries::get_session_title(&conn, "session-1").unwrap();
    assert_eq!(title, None);
}

#[test]
fn test_get_session_title_nonexistent_session() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    let title = queries::get_session_title(&conn, "nonexistent").unwrap();
    assert_eq!(title, None);
}

#[test]
fn test_delete_old_events() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    // Setup device and session
    let now = chrono::Utc::now().to_rfc3339();
    queries::upsert_device(&conn, "device-1", "Device", "macos", &now).unwrap();
    queries::upsert_session(&conn, "session-1", "device-1", &now, None, None, None).unwrap();

    // Insert old event (8 days ago)
    let old_time = (chrono::Utc::now() - chrono::Duration::days(8))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    queries::insert_event(
        &conn,
        "device-1",
        "session-1",
        "tool-use",
        &old_time,
        &old_time,
        None,
        None,
        "{}",
    )
    .unwrap();

    // Insert recent event
    let recent = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    queries::insert_event(
        &conn,
        "device-1",
        "session-1",
        "tool-use",
        &recent,
        &recent,
        None,
        None,
        "{}",
    )
    .unwrap();

    // Delete events older than 7 days
    let deleted = queries::delete_old_events(&conn, 7).unwrap();
    assert_eq!(deleted, 1);

    // Verify only recent event remains
    let events = queries::list_events(&conn, "session-1", 10).unwrap();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_delete_stale_sessions() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    queries::upsert_device(&conn, "device-1", "Device", "macos", &now).unwrap();

    // Insert old orphaned session (8 days ago)
    let old_time = (chrono::Utc::now() - chrono::Duration::days(8)).to_rfc3339();
    queries::upsert_session(
        &conn,
        "old-session",
        "device-1",
        &old_time,
        Some("ended"),
        None,
        None,
    )
    .unwrap();

    // Insert recent session
    queries::upsert_session(
        &conn,
        "recent-session",
        "device-1",
        &now,
        Some("active"),
        None,
        None,
    )
    .unwrap();

    // Delete stale sessions older than 7 days
    let deleted = queries::delete_stale_sessions(&conn, 7).unwrap();
    assert_eq!(deleted, 1);

    // Verify only recent session remains
    let sessions = queries::list_sessions(&conn, "device-1", None, 10).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, "recent-session");
}

#[test]
fn test_delete_stale_sessions_keeps_session_with_events() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    queries::upsert_device(&conn, "device-1", "Device", "macos", &now).unwrap();

    // Insert old session (8 days ago)
    let old_time = (chrono::Utc::now() - chrono::Duration::days(8)).to_rfc3339();
    queries::upsert_session(
        &conn,
        "old-session-with-events",
        "device-1",
        &old_time,
        Some("ended"),
        None,
        None,
    )
    .unwrap();

    // Insert an event referencing this session (recent event, so not cleaned by delete_old_events)
    let recent = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    queries::insert_event(
        &conn,
        "device-1",
        "old-session-with-events",
        "tool-use",
        &recent,
        &recent,
        None,
        None,
        "{}",
    )
    .unwrap();

    // Try to delete stale sessions — should NOT delete because events still reference it
    let deleted = queries::delete_stale_sessions(&conn, 7).unwrap();
    assert_eq!(deleted, 0);

    let sessions = queries::list_sessions(&conn, "device-1", None, 10).unwrap();
    assert_eq!(sessions.len(), 1);
}

#[test]
fn test_delete_stale_devices() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    // Insert old orphaned device (31 days ago)
    let old_time = (chrono::Utc::now() - chrono::Duration::days(31)).to_rfc3339();
    queries::upsert_device(&conn, "old-device", "Old Device", "macos", &old_time).unwrap();

    // Insert recent device
    let now = chrono::Utc::now().to_rfc3339();
    queries::upsert_device(&conn, "recent-device", "Recent Device", "linux", &now).unwrap();

    // Delete stale devices older than 30 days
    let deleted = queries::delete_stale_devices(&conn, 30).unwrap();
    assert_eq!(deleted, 1);

    // Verify only recent device remains
    let devices = queries::list_devices(&conn).unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].device_id, "recent-device");
}

#[test]
fn test_delete_stale_devices_keeps_device_with_sessions() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    // Insert old device (31 days ago)
    let old_time = (chrono::Utc::now() - chrono::Duration::days(31)).to_rfc3339();
    queries::upsert_device(&conn, "old-device", "Old Device", "macos", &old_time).unwrap();

    // Add a session referencing this device
    let now = chrono::Utc::now().to_rfc3339();
    queries::upsert_session(
        &conn,
        "session-on-old-device",
        "old-device",
        &now,
        Some("active"),
        None,
        None,
    )
    .unwrap();

    // Try to delete stale devices — should NOT delete because sessions still reference it
    let deleted = queries::delete_stale_devices(&conn, 30).unwrap();
    assert_eq!(deleted, 0);

    let devices = queries::list_devices(&conn).unwrap();
    assert_eq!(devices.len(), 1);
}

#[test]
fn test_full_retention_cascade() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    // --- Old chain (should be fully cleaned) ---
    let old_time = (chrono::Utc::now() - chrono::Duration::days(31)).to_rfc3339();
    let old_time_millis = (chrono::Utc::now() - chrono::Duration::days(31))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    queries::upsert_device(&conn, "old-device", "Old", "macos", &old_time).unwrap();
    queries::upsert_session(
        &conn,
        "old-session",
        "old-device",
        &old_time,
        Some("ended"),
        None,
        None,
    )
    .unwrap();
    queries::insert_event(
        &conn,
        "old-device",
        "old-session",
        "tool-use",
        &old_time_millis,
        &old_time_millis,
        None,
        None,
        "{}",
    )
    .unwrap();

    // --- Recent chain (should be untouched) ---
    let now = chrono::Utc::now().to_rfc3339();
    let now_millis = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    queries::upsert_device(&conn, "new-device", "New", "linux", &now).unwrap();
    queries::upsert_session(
        &conn,
        "new-session",
        "new-device",
        &now,
        Some("active"),
        None,
        None,
    )
    .unwrap();
    queries::insert_event(
        &conn,
        "new-device",
        "new-session",
        "tool-use",
        &now_millis,
        &now_millis,
        None,
        None,
        "{}",
    )
    .unwrap();

    // Execute cleanup in FK-safe order
    let events_deleted = queries::delete_old_events(&conn, 7).unwrap();
    assert_eq!(events_deleted, 1);

    let sessions_deleted = queries::delete_stale_sessions(&conn, 7).unwrap();
    assert_eq!(sessions_deleted, 1);

    let devices_deleted = queries::delete_stale_devices(&conn, 30).unwrap();
    assert_eq!(devices_deleted, 1);

    // Verify recent chain is untouched
    let devices = queries::list_devices(&conn).unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].device_id, "new-device");

    let sessions = queries::list_sessions(&conn, "new-device", None, 10).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, "new-session");

    let events = queries::list_events(&conn, "new-session", 10).unwrap();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_api_key_insert_and_list() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    queries::insert_api_key(
        &conn,
        "key-id-1",
        "hook-client",
        "claud_abc123",
        "write",
        &now,
    )
    .unwrap();

    let keys = queries::list_api_keys(&conn).unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].id, "key-id-1");
    assert_eq!(keys[0].name, "hook-client");
    assert_eq!(keys[0].key, "claud_abc123");
    assert_eq!(keys[0].scopes, "write");
    assert_eq!(keys[0].created_at, now);
    assert!(keys[0].last_used.is_none());
}

#[test]
fn test_api_key_list_empty() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let keys = queries::list_api_keys(&conn).unwrap();
    assert!(keys.is_empty());
}

#[test]
fn test_api_key_list_multiple_ordered_by_created_at() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    let t1 = "2024-01-01T10:00:00Z";
    let t2 = "2024-01-01T11:00:00Z";
    let t3 = "2024-01-01T12:00:00Z";

    // Insert in non-sequential order
    queries::insert_api_key(&conn, "id-2", "second", "claud_key2", "read", t2).unwrap();
    queries::insert_api_key(&conn, "id-1", "first", "claud_key1", "write", t1).unwrap();
    queries::insert_api_key(&conn, "id-3", "third", "claud_key3", "read,write", t3).unwrap();

    let keys = queries::list_api_keys(&conn).unwrap();
    assert_eq!(keys.len(), 3);
    assert_eq!(keys[0].id, "id-1");
    assert_eq!(keys[1].id, "id-2");
    assert_eq!(keys[2].id, "id-3");
}

#[test]
fn test_api_key_find_by_key_existing() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    queries::insert_api_key(&conn, "id-1", "ios-app", "claud_findme", "read", &now).unwrap();

    let result = queries::find_api_key_by_key(&conn, "claud_findme").unwrap();
    assert!(result.is_some());
    let row = result.unwrap();
    assert_eq!(row.id, "id-1");
    assert_eq!(row.name, "ios-app");
    assert_eq!(row.scopes, "read");
}

#[test]
fn test_api_key_find_by_key_not_found() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    let result = queries::find_api_key_by_key(&conn, "claud_doesnotexist").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_api_key_delete() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    queries::insert_api_key(&conn, "id-1", "to-delete", "claud_deleteme", "read", &now).unwrap();

    let keys_before = queries::list_api_keys(&conn).unwrap();
    assert_eq!(keys_before.len(), 1);

    queries::delete_api_key(&conn, "id-1").unwrap();

    let keys_after = queries::list_api_keys(&conn).unwrap();
    assert!(keys_after.is_empty());
}

#[test]
fn test_api_key_delete_nonexistent_is_ok() {
    let pool = test_pool();
    let conn = pool.get().unwrap();

    // Should not error
    queries::delete_api_key(&conn, "nonexistent-id").unwrap();
}

#[test]
fn test_api_key_update_last_used() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    queries::insert_api_key(&conn, "id-1", "test", "claud_key", "read,write", &now).unwrap();

    // Initially null
    let row = queries::find_api_key_by_key(&conn, "claud_key")
        .unwrap()
        .unwrap();
    assert!(row.last_used.is_none());

    // Update
    let used_at = "2024-06-01T12:00:00Z";
    queries::update_api_key_last_used(&conn, "id-1", used_at).unwrap();

    let row = queries::find_api_key_by_key(&conn, "claud_key")
        .unwrap()
        .unwrap();
    assert_eq!(row.last_used.as_deref(), Some(used_at));
}

#[test]
fn test_api_key_unique_key_constraint() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    queries::insert_api_key(&conn, "id-1", "first", "claud_sameval", "read", &now).unwrap();

    // Inserting a second key with the same `key` value should fail
    let result = queries::insert_api_key(&conn, "id-2", "second", "claud_sameval", "write", &now);
    assert!(result.is_err());
}

#[test]
fn test_api_key_scopes_comma_separated() {
    let pool = test_pool();
    let conn = pool.get().unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    queries::insert_api_key(&conn, "id-1", "rw", "claud_rw", "read,write", &now).unwrap();

    let row = queries::find_api_key_by_key(&conn, "claud_rw")
        .unwrap()
        .unwrap();
    assert_eq!(row.scopes, "read,write");
}
