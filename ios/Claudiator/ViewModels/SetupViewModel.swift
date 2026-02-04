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
        if !url.hasPrefix("http") {
            let isLocal = url.contains(".local") || url.starts(with: "localhost") || url.starts(with: "127.0.0.1")
            url = (isLocal ? "http://" : "https://") + url
        }

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
        } catch let error as APIError {
            apiClient.baseURL = oldURL
            try? KeychainService.delete(key: "api_key")
            errorMessage = error.errorDescription
        } catch let urlError as URLError {
            apiClient.baseURL = oldURL
            try? KeychainService.delete(key: "api_key")
            switch urlError.code {
            case .cannotConnectToHost, .cannotFindHost:
                errorMessage = "Cannot reach server. Check the URL and ensure the server is running."
            case .timedOut:
                errorMessage = "Connection timed out. Check your network and server address."
            case .secureConnectionFailed, .appTransportSecurityRequiresSecureConnection:
                errorMessage = "Secure connection failed. For local servers, try using http:// explicitly."
            case .notConnectedToInternet:
                errorMessage = "No internet connection."
            default:
                errorMessage = "Network error: \(urlError.localizedDescription)"
            }
        } catch {
            apiClient.baseURL = oldURL
            try? KeychainService.delete(key: "api_key")
            errorMessage = "Connection failed: \(error.localizedDescription)"
        }
        isLoading = false
    }
}
