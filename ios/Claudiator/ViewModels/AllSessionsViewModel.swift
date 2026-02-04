import Foundation
import Observation

@Observable
class AllSessionsViewModel {
    var sessions: [Session] = []
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
            let allSessions = try await apiClient.fetchAllSessions()
            if filter == .active {
                sessions = allSessions.filter { $0.status != "ended" }
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
