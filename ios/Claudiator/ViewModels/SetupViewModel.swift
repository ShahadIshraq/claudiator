import Foundation
import Observation

@Observable
class SetupViewModel {
    var serverURL: String = ""
    var apiKey: String = ""
    var isLoading = false
    var errorMessage: String?

    func connect(apiClient: APIClient) async {
        // Clean URL
        var url = serverURL.trimmingCharacters(in: .whitespacesAndNewlines)
        if url.hasSuffix("/") { url.removeLast() }
        if !url.hasPrefix("http") { url = "https://" + url }

        guard URL(string: url) != nil else {
            errorMessage = "Invalid URL format"
            return
        }

        isLoading = true
        errorMessage = nil

        // Temporarily configure to test
        let oldURL = apiClient.baseURL
        do {
            try apiClient.configure(url: url, apiKey: apiKey)
            _ = try await apiClient.ping()
            // Success - config is saved
        } catch {
            // Restore old config on failure
            apiClient.baseURL = oldURL
            try? KeychainService.delete(key: "api_key")
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
