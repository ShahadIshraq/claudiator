import Foundation
import Observation

@MainActor
@Observable
class AllSessionsViewModel {
    var sessions: [Session] = []
    var isLoading = false
    var isLoadingMore = false
    var hasMore = false
    var currentOffset = 0
    var errorMessage: String?
    var filter: SessionFilter = .active

    // Grouping state
    var isGroupedByDevice: Bool = false
    var expandedDevices: Set<String> = []
    private(set) var groupedSessions: [String: [Session]] = [:]

    var apiClient: APIClient?

    private static let groupingKey = "sessionsGroupedByDevice"

    enum SessionFilter: String, CaseIterable {
        case active = "Active"
        case all = "All"
    }

    init(apiClient: APIClient? = nil) {
        self.apiClient = apiClient
        isGroupedByDevice = UserDefaults.standard.bool(forKey: Self.groupingKey)
    }

    func refresh() async {
        guard let apiClient else { return }
        await refresh(apiClient: apiClient)
    }

    func refresh(apiClient: APIClient) async {
        if sessions.isEmpty { isLoading = true }
        do {
            let excludeEnded = (filter == .active)
            let result = try await apiClient.fetchAllSessionsPage(excludeEnded: excludeEnded, limit: 50, offset: 0)
            sessions = result.sessions
            hasMore = result.hasMore
            currentOffset = result.nextOffset
            errorMessage = nil

            if isGroupedByDevice {
                groupSessions()
            }
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    func loadMore(apiClient: APIClient) async {
        guard hasMore, !isLoadingMore else { return }
        isLoadingMore = true
        do {
            let excludeEnded = (filter == .active)
            let result = try await apiClient.fetchAllSessionsPage(excludeEnded: excludeEnded, limit: 50, offset: currentOffset)
            // Deduplicate by sessionId to guard against list drift
            let existingIds = Set(sessions.map(\.sessionId))
            let newSessions = result.sessions.filter { !existingIds.contains($0.sessionId) }
            sessions.append(contentsOf: newSessions)
            hasMore = result.hasMore
            currentOffset = result.nextOffset
            errorMessage = nil

            if isGroupedByDevice {
                groupSessions()
            }
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoadingMore = false
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

        if isGroupedByDevice {
            expandedDevices = Set(
                groupedSessions.compactMap { deviceId, sessions in
                    sessions.contains { $0.status != "ended" } ? deviceId : nil
                }
            )
        }
    }
}
