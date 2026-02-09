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
        .query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table'", [], |row| row.get(0))
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
    assert_eq!(queries::get_metadata(&conn, "key1").unwrap(), Some("value1".to_string()));
    assert_eq!(queries::get_metadata(&conn, "key2").unwrap(), Some("value2".to_string()));
    assert_eq!(queries::get_metadata(&conn, "key3").unwrap(), Some("value3".to_string()));
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
        &["notif-1".to_string(), "notif-2".to_string(), "notif-3".to_string()],
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
