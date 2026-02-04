import Foundation
import Observation

@Observable
class AllSessionsViewModel {
    var sessions: [(session: Session, deviceName: String, platform: String)] = []
    var isLoading = false
    var errorMessage: String?
    var filter: SessionFilter = .active

    enum SessionFilter: String, CaseIterable {
        case active = "Active"
        case all = "All"
    }

    func refresh(apiClient: APIClient) async {
        if sessions.isEmpty { isLoading = true }
        do {
            let devices = try await apiClient.fetchDevices()
            var allSessions: [(session: Session, deviceName: String, platform: String)] = []
            for device in devices {
                let deviceSessions = try await apiClient.fetchSessions(deviceId: device.deviceId)
                for s in deviceSessions {
                    allSessions.append((session: s, deviceName: device.deviceName, platform: device.platform))
                }
            }
            // Sort by lastEvent descending
            allSessions.sort { $0.session.lastEvent > $1.session.lastEvent }

            if filter == .active {
                sessions = allSessions.filter { $0.session.status != "ended" }
            } else {
                sessions = allSessions
            }
            errorMessage = nil
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
