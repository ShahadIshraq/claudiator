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
            async let fetchedDevices = apiClient.fetchDevices()
            async let fetchedSessions = apiClient.fetchAllSessions()
            let (devices, allSessions) = try await (fetchedDevices, fetchedSessions)

            let deviceMap = Dictionary(uniqueKeysWithValues: devices.map { ($0.deviceId, $0) })

            var combined: [(session: Session, deviceName: String, platform: String)] = []
            for s in allSessions {
                let device = deviceMap[s.deviceId]
                combined.append((
                    session: s,
                    deviceName: device?.deviceName ?? "Unknown",
                    platform: device?.platform ?? "unknown"
                ))
            }

            combined.sort { $0.session.lastEvent > $1.session.lastEvent }

            if filter == .active {
                sessions = combined.filter { $0.session.status != "ended" }
            } else {
                sessions = combined
            }
            errorMessage = nil
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
