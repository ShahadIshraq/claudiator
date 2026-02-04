import Foundation
import Observation

struct SessionStatusCounts {
    var active: Int = 0
    var waitingInput: Int = 0
    var waitingPermission: Int = 0
    var idle: Int = 0
    var ended: Int = 0

    var totalActive: Int { active + waitingInput + waitingPermission + idle }
}

@Observable
class DeviceListViewModel {
    var devices: [Device] = []
    var statusCounts: [String: SessionStatusCounts] = [:]
    var isLoading = false
    var errorMessage: String?

    func refresh(apiClient: APIClient) async {
        if devices.isEmpty { isLoading = true }
        do {
            let fetchedDevices = try await apiClient.fetchDevices()
            var counts: [String: SessionStatusCounts] = [:]
            for device in fetchedDevices {
                let sessions = try await apiClient.fetchSessions(deviceId: device.deviceId)
                var c = SessionStatusCounts()
                for s in sessions {
                    switch s.status {
                    case "active": c.active += 1
                    case "waiting_for_input": c.waitingInput += 1
                    case "waiting_for_permission": c.waitingPermission += 1
                    case "idle": c.idle += 1
                    case "ended": c.ended += 1
                    default: break
                    }
                }
                counts[device.deviceId] = c
            }
            devices = fetchedDevices
            statusCounts = counts
            errorMessage = nil
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
