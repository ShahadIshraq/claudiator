import Foundation
import Observation

@MainActor
@Observable
class AllSessionsViewModel {
    var sessions: [Session] = []
    var isLoading = false
    var errorMessage: String?
    var filter: SessionFilter = .active

    // Grouping state
    var isGroupedByDevice: Bool = false
    var expandedDevices: Set<String> = []
    private(set) var groupedSessions: [String: [Session]] = [:]

    private static let groupingKey = "sessionsGroupedByDevice"

    enum SessionFilter: String, CaseIterable {
        case active = "Active"
        case all = "All"
    }

    init() {
        // Load persisted grouping preference
        isGroupedByDevice = UserDefaults.standard.bool(forKey: Self.groupingKey)
    }

    func refresh(apiClient: APIClient) async {
        if sessions.isEmpty { isLoading = true }
        do {
            let allSessions = try await apiClient.fetchAllSessions()
            if filter == .active {
                sessions = allSessions.filter { $0.status != "ended" }
            } else {
                sessions = allSessions
            }
            errorMessage = nil

            // Only group when grouping is enabled
            if isGroupedByDevice {
                groupSessions()
            }
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    func toggleGrouping() {
        isGroupedByDevice.toggle()
        UserDefaults.standard.set(isGroupedByDevice, forKey: Self.groupingKey)
        if isGroupedByDevice {
            groupSessions()
        }
    }

    func toggleDevice(_ deviceId: String) {
        if expandedDevices.contains(deviceId) {
            expandedDevices.remove(deviceId)
        } else {
            expandedDevices.insert(deviceId)
        }
    }

    private func groupSessions() {
        groupedSessions = Dictionary(grouping: sessions) { session in
            session.deviceId
        }

        // Auto-expand devices with active sessions when grouping is enabled
        if isGroupedByDevice {
            expandedDevices = Set(
                groupedSessions.compactMap { deviceId, sessions in
                    sessions.contains { $0.status != "ended" } ? deviceId : nil
                }
            )
        }
    }
}
