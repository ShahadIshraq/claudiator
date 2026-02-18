import Foundation
import Observation

@MainActor
@Observable
class EventListViewModel {
    var events: [Event] = []
    var isLoading = false
    var errorMessage: String?

    var apiClient: APIClient?
    var sessionId: String?

    init(apiClient: APIClient? = nil) {
        self.apiClient = apiClient
    }

    func refresh() async {
        guard let apiClient, let sessionId else { return }
        await refresh(apiClient: apiClient, sessionId: sessionId)
    }

    func refresh(apiClient: APIClient, sessionId: String) async {
        if events.isEmpty { isLoading = true }
        do {
            events = try await apiClient.fetchEvents(sessionId: sessionId)
            errorMessage = nil
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
