import Foundation
import Observation

@Observable
class EventListViewModel {
    var events: [Event] = []
    var isLoading = false
    var errorMessage: String?

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
