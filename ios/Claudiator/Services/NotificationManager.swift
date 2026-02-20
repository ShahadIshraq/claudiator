import Foundation
import UserNotifications

@MainActor
@Observable
class NotificationManager {
    var unreadNotifications: [AppNotification] = []
    var allNotifications: [AppNotification] = []

    var unreadCount: Int {
        unreadNotifications.count
    }

    var sessionsWithNotifications: Set<String> {
        Set(unreadNotifications.map(\.sessionId))
    }

    private let userDefaults = UserDefaults.standard
    private let lastSeenKey = "lastSeenTimestamp"
    private let readIdsKey = "readNotificationIds"
    private let pushReceivedKey = "pushReceivedNotificationIds"
    private let pushReceivedTimestampsKey = "pushReceivedTimestamps"
    private let maxNotifications = 100
    private let maxReadIds = 500
    private let pushReceivedRetentionSeconds: TimeInterval = 600 // 10 minutes

    init() {
        loadFromStorage()
    }

    // MARK: - Public Methods

    func fetchNewNotifications(apiClient: APIClient) async {
        do {
            let lastSeen = userDefaults.string(forKey: lastSeenKey)
            let notifications = try await apiClient.fetchNotifications(after: lastSeen, limit: nil)

            guard !notifications.isEmpty else { return }

            // Update last seen timestamp to the most recent notification's timestamp
            // Notifications are returned in ascending order, so .last is the newest
            if let mostRecent = notifications.last {
                userDefaults.set(mostRecent.createdAt, forKey: lastSeenKey)
            }

            // Get current read notification IDs and push-received IDs
            var readIds = getReadNotificationIds()
            let pushReceivedIds = getPushReceivedIds()

            // Seed read state from server's acknowledged field
            let serverAcknowledgedIds = notifications.filter { $0.acknowledged == true }.map(\.notificationId)
            readIds.formUnion(serverAcknowledgedIds)
            if !serverAcknowledgedIds.isEmpty {
                saveReadNotificationIds(readIds)
            }

            // Fire local notification ONLY for the most recent unread notification
            // SKIP if it was already received via APNs push (deduplication)
            if let mostRecent = notifications.first,
               !readIds.contains(mostRecent.notificationId),
               !pushReceivedIds.contains(mostRecent.notificationId) {
                await fireLocalNotification(mostRecent)
            }

            // Update internal state
            // Add new notifications to all notifications
            let newNotifications = notifications.filter { notif in
                !allNotifications.contains(where: { $0.notificationId == notif.notificationId })
            }
            allNotifications.insert(contentsOf: newNotifications, at: 0)

            // Cap at 100 entries
            if allNotifications.count > maxNotifications {
                allNotifications = Array(allNotifications.prefix(maxNotifications))
            }

            // Update unread notifications
            unreadNotifications = allNotifications.filter { !readIds.contains($0.notificationId) }
        } catch {
            print("Error fetching notifications: \(error)")
        }
    }

    func markSessionRead(sessionId: String, apiClient: APIClient) async {
        let notificationsToMark = unreadNotifications.filter { $0.sessionId == sessionId }
        let idsToMark = notificationsToMark.map(\.notificationId)

        guard !idsToMark.isEmpty else { return }

        // Update read IDs
        var readIds = getReadNotificationIds()
        readIds.formUnion(idsToMark)
        saveReadNotificationIds(readIds)

        // Update unread notifications
        unreadNotifications.removeAll { $0.sessionId == sessionId }

        // Acknowledge on server
        try? await apiClient.acknowledgeNotifications(ids: idsToMark)
    }

    func markNotificationRead(notificationId: String, apiClient: APIClient) async {
        guard unreadNotifications.contains(where: { $0.notificationId == notificationId }) else {
            return
        }

        // Update read IDs
        var readIds = getReadNotificationIds()
        readIds.insert(notificationId)
        saveReadNotificationIds(readIds)

        // Update unread notifications
        unreadNotifications.removeAll { $0.notificationId == notificationId }

        // Acknowledge on server
        try? await apiClient.acknowledgeNotifications(ids: [notificationId])
    }

    func markAllRead(apiClient: APIClient) async {
        let ids = unreadNotifications.map { $0.notificationId }
        guard !ids.isEmpty else { return }

        // Update read IDs
        var readIds = getReadNotificationIds()
        readIds.formUnion(ids)
        saveReadNotificationIds(readIds)

        // Update unread notifications
        unreadNotifications.removeAll()

        // Acknowledge on server
        try? await apiClient.acknowledgeNotifications(ids: ids)
    }

    // MARK: - Private Methods

    private func fireLocalNotification(_ notif: AppNotification) async {
        let content = UNMutableNotificationContent()
        content.title = notif.title
        content.body = notif.body
        content.sound = .default
        content.userInfo = [
            "notification_id": notif.notificationId,
            "session_id": notif.sessionId,
            "device_id": notif.deviceId
        ]

        let request = UNNotificationRequest(
            identifier: notif.notificationId,
            content: content,
            trigger: nil
        )

        try? await UNUserNotificationCenter.current().add(request)
    }

    private func loadFromStorage() {
        let readIds = getReadNotificationIds()
        unreadNotifications = allNotifications.filter { !readIds.contains($0.notificationId) }
    }

    private func getReadNotificationIds() -> Set<String> {
        if let array = userDefaults.array(forKey: readIdsKey) as? [String] {
            return Set(array)
        }
        return Set()
    }

    private func saveReadNotificationIds(_ ids: Set<String>) {
        var trimmedIds = ids
        // Trim to maxReadIds if exceeds the cap
        if trimmedIds.count > maxReadIds {
            // Keep the most recent maxReadIds entries
            trimmedIds = Set(Array(trimmedIds.sorted().suffix(maxReadIds)))
        }
        userDefaults.set(Array(trimmedIds), forKey: readIdsKey)
    }

    private func getPushReceivedIds() -> Set<String> {
        // Clean up old entries (older than 1 hour)
        cleanupOldPushReceivedIds()

        if let array = userDefaults.array(forKey: pushReceivedKey) as? [String] {
            return Set(array)
        }
        return Set()
    }

    private func savePushReceivedIds(_ ids: Set<String>) {
        userDefaults.set(Array(ids), forKey: pushReceivedKey)
    }

    private func cleanupOldPushReceivedIds() {
        guard var timestamps = userDefaults.dictionary(forKey: pushReceivedTimestampsKey) as? [String: TimeInterval] else {
            return
        }

        let now = Date().timeIntervalSince1970
        let cutoff = now - pushReceivedRetentionSeconds

        // Remove IDs older than retention period
        let idsToRemove = timestamps.filter { $0.value < cutoff }.map(\.key)

        if !idsToRemove.isEmpty {
            var currentIds = Set(userDefaults.array(forKey: pushReceivedKey) as? [String] ?? [])
            for id in idsToRemove {
                currentIds.remove(id)
                timestamps.removeValue(forKey: id)
            }

            userDefaults.set(Array(currentIds), forKey: pushReceivedKey)
            userDefaults.set(timestamps, forKey: pushReceivedTimestampsKey)
        }
    }

    /// Called when an APNs push notification is received
    func markReceivedViaPush(notificationId: String) {
        cleanupOldPushReceivedIds()

        var ids = Set(userDefaults.array(forKey: pushReceivedKey) as? [String] ?? [])
        ids.insert(notificationId)
        savePushReceivedIds(ids)

        // Store timestamp for cleanup
        var timestamps = userDefaults.dictionary(forKey: pushReceivedTimestampsKey) as? [String: TimeInterval] ?? [:]
        timestamps[notificationId] = Date().timeIntervalSince1970
        userDefaults.set(timestamps, forKey: pushReceivedTimestampsKey)
    }
}
