import Foundation
import Observation

@MainActor
@Observable
class SessionListViewModel {
    var sessions: [Session] = []
    var isLoading = false
    var errorMessage: String?
    var filter: SessionFilter = .active

    var apiClient: APIClient?
    var deviceId: String?

    enum SessionFilter: String, CaseIterable {
        case active = "Active"
        case all = "All"
    }

    init(apiClient: APIClient? = nil) {
        self.apiClient = apiClient
    }

    func refresh() async {
        guard let apiClient, let deviceId else { return }
        await refresh(apiClient: apiClient, deviceId: deviceId)
    }

    func refresh(apiClient: APIClient, deviceId: String) async {
        if sessions.isEmpty { isLoading = true }
        do {
            // For "active" filter, exclude ended sessions client-side
            let all = try await apiClient.fetchSessions(deviceId: deviceId)
            if filter == .active {
                sessions = all.filter { $0.status != "ended" }
            } else {
                sessions = all
            }
            errorMessage = nil
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
