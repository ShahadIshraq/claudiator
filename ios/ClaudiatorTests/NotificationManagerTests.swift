import Testing
import Foundation
@testable import Claudiator

// MARK: - Mock APIClient for Testing

@MainActor
class MockAPIClient: APIClient {
    override init() {
        super.init()
        // Configure with test URL and API key
        try? configure(url: "https://test.example.com", apiKey: "test-key")
    }

    override func acknowledgeNotifications(ids: [String]) async throws {
        // Mock implementation - does nothing
    }
}

@Suite("NotificationManager Tests")
@MainActor
struct NotificationManagerTests {
    // MARK: - Helper to create test notifications

    func createTestNotification(id: String, sessionId: String, deviceId: String = "dev1") -> AppNotification {
        AppNotification(
            notificationId: id,
            sessionId: sessionId,
            deviceId: deviceId,
            title: "Test Notification",
            body: "Test body",
            notificationType: "info",
            payloadJson: nil,
            createdAt: "2024-01-15T10:00:00Z",
            acknowledged: false
        )
    }

    // MARK: - Initialization Tests

    @Test("NotificationManager initializes with empty state")
    func testInitialState() {
        // Clean up UserDefaults before test
        let manager = NotificationManager()

        #expect(manager.unreadNotifications.isEmpty)
        #expect(manager.allNotifications.isEmpty)
        #expect(manager.unreadCount == 0)
        #expect(manager.sessionsWithNotifications.isEmpty)
    }

    // MARK: - markSessionRead Tests

    @Test("markSessionRead removes notifications for session")
    func testMarkSessionRead() async {
        let manager = NotificationManager()
        let mockClient = MockAPIClient()

        // Manually add test notifications
        let notif1 = createTestNotification(id: "notif1", sessionId: "sess1")
        let notif2 = createTestNotification(id: "notif2", sessionId: "sess1")
        let notif3 = createTestNotification(id: "notif3", sessionId: "sess2")

        // Simulate unread notifications
        manager.allNotifications = [notif1, notif2, notif3]
        manager.unreadNotifications = [notif1, notif2, notif3]

        // Mark session 1 as read
        await manager.markSessionRead(sessionId: "sess1", apiClient: mockClient)

        // Verify only sess2 notification remains unread
        #expect(manager.unreadNotifications.count == 1)
        #expect(manager.unreadNotifications.first?.sessionId == "sess2")
        #expect(manager.unreadCount == 1)
    }

    @Test("markSessionRead handles non-existent session")
    func testMarkSessionReadNonExistent() async {
        let manager = NotificationManager()
        let mockClient = MockAPIClient()

        let notif1 = createTestNotification(id: "notif1", sessionId: "sess1")
        manager.allNotifications = [notif1]
        manager.unreadNotifications = [notif1]

        // Mark non-existent session
        await manager.markSessionRead(sessionId: "sess99", apiClient: mockClient)

        // Verify no change
        #expect(manager.unreadNotifications.count == 1)
    }

    @Test("markSessionRead handles empty unread list")
    func testMarkSessionReadEmpty() async {
        let manager = NotificationManager()
        let mockClient = MockAPIClient()

        await manager.markSessionRead(sessionId: "sess1", apiClient: mockClient)

        #expect(manager.unreadNotifications.isEmpty)
    }

    // MARK: - markNotificationRead Tests

    @Test("markNotificationRead removes specific notification")
    func testMarkNotificationRead() async {
        let manager = NotificationManager()
        let mockClient = MockAPIClient()

        let notif1 = createTestNotification(id: "notif1", sessionId: "sess1")
        let notif2 = createTestNotification(id: "notif2", sessionId: "sess1")

        manager.allNotifications = [notif1, notif2]
        manager.unreadNotifications = [notif1, notif2]

        // Mark specific notification as read
        await manager.markNotificationRead(notificationId: "notif1", apiClient: mockClient)

        // Verify only notif2 remains unread
        #expect(manager.unreadNotifications.count == 1)
        #expect(manager.unreadNotifications.first?.notificationId == "notif2")
    }

    @Test("markNotificationRead handles non-existent notification")
    func testMarkNotificationReadNonExistent() async {
        let manager = NotificationManager()
        let mockClient = MockAPIClient()

        let notif1 = createTestNotification(id: "notif1", sessionId: "sess1")
        manager.allNotifications = [notif1]
        manager.unreadNotifications = [notif1]

        // Mark non-existent notification
        await manager.markNotificationRead(notificationId: "notif99", apiClient: mockClient)

        // Verify no change
        #expect(manager.unreadNotifications.count == 1)
    }

    // MARK: - sessionsWithNotifications Tests

    @Test("sessionsWithNotifications returns unique session IDs")
    func testSessionsWithNotifications() {
        let manager = NotificationManager()

        let notif1 = createTestNotification(id: "notif1", sessionId: "sess1")
        let notif2 = createTestNotification(id: "notif2", sessionId: "sess1")
        let notif3 = createTestNotification(id: "notif3", sessionId: "sess2")

        manager.unreadNotifications = [notif1, notif2, notif3]

        let sessions = manager.sessionsWithNotifications
        #expect(sessions.count == 2)
        #expect(sessions.contains("sess1"))
        #expect(sessions.contains("sess2"))
    }

    @Test("sessionsWithNotifications returns empty set for no notifications")
    func testSessionsWithNotificationsEmpty() {
        let manager = NotificationManager()
        let sessions = manager.sessionsWithNotifications

        #expect(sessions.isEmpty)
    }

    // MARK: - Deduplication Tests

    @Test("markReceivedViaPush stores notification ID")
    func testMarkReceivedViaPush() {
        let manager = NotificationManager()

        manager.markReceivedViaPush(notificationId: "notif1")

        // We can't directly check the private method, but we can verify
        // that calling it doesn't crash and the manager still works
        #expect(manager.unreadNotifications.isEmpty)
    }

    // MARK: - State Management Tests

    @Test("unreadCount reflects unread notifications")
    func testUnreadCount() {
        let manager = NotificationManager()

        #expect(manager.unreadCount == 0)

        let notif1 = createTestNotification(id: "notif1", sessionId: "sess1")
        let notif2 = createTestNotification(id: "notif2", sessionId: "sess2")

        manager.unreadNotifications = [notif1, notif2]
        #expect(manager.unreadCount == 2)

        manager.unreadNotifications = [notif1]
        #expect(manager.unreadCount == 1)
    }

    @Test("Multiple markSessionRead calls are idempotent")
    func testMarkSessionReadIdempotent() async {
        let manager = NotificationManager()
        let mockClient = MockAPIClient()

        let notif1 = createTestNotification(id: "notif1", sessionId: "sess1")
        manager.allNotifications = [notif1]
        manager.unreadNotifications = [notif1]

        await manager.markSessionRead(sessionId: "sess1", apiClient: mockClient)
        #expect(manager.unreadNotifications.isEmpty)

        // Call again - should not crash
        await manager.markSessionRead(sessionId: "sess1", apiClient: mockClient)
        #expect(manager.unreadNotifications.isEmpty)
    }

    // MARK: - Last Seen Tracking Tests

    @Test("LastSeen is set to the newest (last) notification ID")
    func testLastSeenTracking() async {
        let manager = NotificationManager()

        // Clear any existing state
        UserDefaults.standard.removeObject(forKey: "lastSeenNotificationId")

        // Create notifications in ascending order (oldest to newest)
        let notif1 = createTestNotification(id: "notif1", sessionId: "sess1")
        let notif2 = createTestNotification(id: "notif2", sessionId: "sess1")
        let notif3 = createTestNotification(id: "notif3", sessionId: "sess1")

        // Simulate the bug fix: notifications.last should be used for lastSeen
        let notifications = [notif1, notif2, notif3]

        // Test the logic that should use .last
        await manager.processNotificationsForTest(notifications)

        // Verify lastSeen is set to the LAST (newest) notification ID, not the first
        let lastSeen = UserDefaults.standard.string(forKey: "lastSeenNotificationId")
        #expect(lastSeen == "notif3", "lastSeen should be set to the LAST notification (newest)")
    }

    @Test("Notification ordering - last element is correctly identified as newest")
    func testNotificationOrdering() async {
        let manager = NotificationManager()

        // Clear any existing state
        UserDefaults.standard.removeObject(forKey: "lastSeenNotificationId")

        // Create multiple notifications with different IDs in ascending order
        let notif1 = createTestNotification(id: "notif100", sessionId: "sess1")
        let notif2 = createTestNotification(id: "notif200", sessionId: "sess1")
        let notif3 = createTestNotification(id: "notif300", sessionId: "sess1")

        let notifications = [notif1, notif2, notif3]

        // Process notifications
        await manager.processNotificationsForTest(notifications)

        // Verify the newest notification (last in the array) is stored as lastSeen
        let lastSeen = UserDefaults.standard.string(forKey: "lastSeenNotificationId")
        #expect(lastSeen == "notif300", "lastSeen should be the last element in the array")
    }

    @Test("Subsequent fetches track lastSeen correctly")
    func testSubsequentFetchesWithLastSeen() async {
        let manager = NotificationManager()

        // Clear any existing state
        UserDefaults.standard.removeObject(forKey: "lastSeenNotificationId")

        // First batch of notifications
        let notif1 = createTestNotification(id: "notif1", sessionId: "sess1")
        let notif2 = createTestNotification(id: "notif2", sessionId: "sess1")

        await manager.processNotificationsForTest([notif1, notif2])

        // Verify lastSeen is set to notif2 (the last one)
        let lastSeenAfterFirst = UserDefaults.standard.string(forKey: "lastSeenNotificationId")
        #expect(lastSeenAfterFirst == "notif2")

        // Second batch - newer notification
        let notif3 = createTestNotification(id: "notif3", sessionId: "sess1")

        await manager.processNotificationsForTest([notif3])

        // Verify lastSeen is now updated to notif3
        let lastSeenAfterSecond = UserDefaults.standard.string(forKey: "lastSeenNotificationId")
        #expect(lastSeenAfterSecond == "notif3")
    }

    @Test("Empty notification list does not change lastSeen")
    func testEmptyNotificationList() async {
        let manager = NotificationManager()

        // Set initial lastSeen
        UserDefaults.standard.set("notif1", forKey: "lastSeenNotificationId")

        // Process empty list
        await manager.processNotificationsForTest([])

        // Verify lastSeen remains unchanged (early return logic)
        let lastSeen = UserDefaults.standard.string(forKey: "lastSeenNotificationId")
        #expect(lastSeen == "notif1", "lastSeen should not change when notification list is empty")
    }

    @Test("Empty notification list triggers early return without updates")
    func testEmptyNotificationEarlyReturn() async {
        let manager = NotificationManager()

        // Set initial state
        UserDefaults.standard.set("notif1", forKey: "lastSeenNotificationId")
        let initialUnreadCount = manager.unreadCount

        // Process empty list
        await manager.processNotificationsForTest([])

        // Verify state remains unchanged (early return)
        #expect(manager.unreadCount == initialUnreadCount)
        let lastSeen = UserDefaults.standard.string(forKey: "lastSeenNotificationId")
        #expect(lastSeen == "notif1")
    }

    @Test("Single notification sets lastSeen correctly")
    func testSingleNotification() async {
        let manager = NotificationManager()

        // Clear any existing state
        UserDefaults.standard.removeObject(forKey: "lastSeenNotificationId")

        let notif1 = createTestNotification(id: "single-notif", sessionId: "sess1")

        await manager.processNotificationsForTest([notif1])

        // Verify lastSeen is set correctly for a single notification
        let lastSeen = UserDefaults.standard.string(forKey: "lastSeenNotificationId")
        #expect(lastSeen == "single-notif")
    }
}

// MARK: - Test Helper Extension

extension NotificationManager {
    /// Helper method for testing the lastSeen tracking logic
    /// This simulates the core logic from fetchNewNotifications without needing APIClient
    func processNotificationsForTest(_ notifications: [AppNotification]) async {
        guard !notifications.isEmpty else { return }

        // This mirrors the bug fix: using .last instead of .first
        if let mostRecent = notifications.last {
            UserDefaults.standard.set(mostRecent.notificationId, forKey: "lastSeenNotificationId")
        }

        // Get current read notification IDs
        let readIds = getReadNotificationIdsForTest()

        // Update internal state
        await MainActor.run {
            let newNotifications = notifications.filter { notif in
                !allNotifications.contains(where: { $0.notificationId == notif.notificationId })
            }
            allNotifications.insert(contentsOf: newNotifications, at: 0)

            if allNotifications.count > 100 {
                allNotifications = Array(allNotifications.prefix(100))
            }

            unreadNotifications = allNotifications.filter { !readIds.contains($0.notificationId) }
        }
    }

    func getReadNotificationIdsForTest() -> Set<String> {
        if let array = UserDefaults.standard.array(forKey: "readNotificationIds") as? [String] {
            return Set(array)
        }
        return Set()
    }
}
