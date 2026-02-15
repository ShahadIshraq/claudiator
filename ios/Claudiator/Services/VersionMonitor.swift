import Foundation
import Observation

@MainActor
@Observable
class VersionMonitor {
    private(set) var dataVersion: UInt64 = 0
    private(set) var notificationVersion: UInt64 = 0
    private var task: Task<Void, Never>?

    func start(apiClient: APIClient, notificationManager: NotificationManager) {
        guard task == nil else { return }
        task = Task {
            while !Task.isCancelled {
                if let versions = try? await apiClient.ping() {
                    let oldNotifVersion = self.notificationVersion
                    self.dataVersion = versions.dataVersion
                    self.notificationVersion = versions.notificationVersion
                    if versions.notificationVersion != oldNotifVersion {
                        await notificationManager.fetchNewNotifications(apiClient: apiClient)
                    }
                }
                try? await Task.sleep(for: .seconds(10))
            }
        }
    }

    func stop() {
        task?.cancel()
        task = nil
    }
}
