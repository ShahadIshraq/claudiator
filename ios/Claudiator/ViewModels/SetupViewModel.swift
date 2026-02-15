import Foundation
import Observation

@MainActor
@Observable
class SetupViewModel {
    var serverURL: String = ""
    var apiKey: String = ""
    var isLoading = false
    var errorMessage: String?
    var connectionSuccess = false

    func connect(apiClient: APIClient) async {
        guard let cleanedURL = cleanAndValidateURL() else {
            errorMessage = "Invalid URL format"
            return
        }

        isLoading = true
        errorMessage = nil

        await testConnection(apiClient: apiClient, url: cleanedURL)
        isLoading = false
    }

    // MARK: - Private Helpers

    private func cleanAndValidateURL() -> String? {
        var url = serverURL.trimmingCharacters(in: .whitespacesAndNewlines)
        if url.hasSuffix("/") { url.removeLast() }
        if !url.hasPrefix("http") {
            let isLocal = url.contains(".local") || url.starts(with: "localhost") || url.starts(with: "127.0.0.1")
            url = (isLocal ? "http://" : "https://") + url
        }

        guard URL(string: url) != nil else { return nil }
        return url
    }

    private func testConnection(apiClient: APIClient, url: String) async {
        let oldURL = apiClient.baseURL
        do {
            try apiClient.configure(url: url, apiKey: apiKey)
            _ = try await apiClient.ping()
            handleConnectionSuccess()
        } catch let error as APIError {
            handleAPIError(error, apiClient: apiClient, oldURL: oldURL)
        } catch let urlError as URLError {
            handleURLError(urlError, apiClient: apiClient, oldURL: oldURL, url: url)
        } catch {
            handleUnexpectedError(error, apiClient: apiClient, oldURL: oldURL)
        }
    }

    private func handleConnectionSuccess() {
        connectionSuccess = true
        NotificationCenter.default.post(name: Notification.Name("RefreshContentView"), object: nil)
    }

    private func handleAPIError(_ error: APIError, apiClient: APIClient, oldURL: String) {
        apiClient.baseURL = oldURL
        try? KeychainService.delete(key: "api_key")

        let message: String
        switch error {
        case .notConfigured:
            message = "❌ Server not configured"
        case .invalidURL:
            message = "❌ Invalid server URL format"
        case .unauthorized:
            message = "❌ Authentication Failed\n\nThe API key is incorrect. Please check your API key and try again."
        case .serverError(let code):
            message = "❌ Server Error (\(code))\n\nThe server returned an error. Make sure the server is running correctly."
        case .networkError(let underlyingError):
            message = "❌ Network Error\n\n\(underlyingError.localizedDescription)"
        case .decodingError(let underlyingError):
            message = "❌ Invalid Response\n\n\(underlyingError.localizedDescription)"
        }

        errorMessage = message
    }

    private func handleURLError(_ urlError: URLError, apiClient: APIClient, oldURL: String, url: String) {
        apiClient.baseURL = oldURL
        try? KeychainService.delete(key: "api_key")

        switch urlError.code {
        case .cannotConnectToHost, .cannotFindHost:
            errorMessage = """
                ❌ Cannot Reach Server

                Unable to connect to: \(url)

                Make sure:
                • The server is running
                • The URL is correct
                • You're on the same network (for local servers)
                """
        case .timedOut:
            errorMessage = "❌ Connection Timeout\n\nThe server didn't respond in time.\n\nCheck:\n• Your network connection\n• The server address"
        case .secureConnectionFailed, .appTransportSecurityRequiresSecureConnection:
            errorMessage = """
                ❌ HTTPS Connection Failed

                For local development servers, use http:// instead of https://

                Example: http://192.168.1.5:3000
                """
        case .notConnectedToInternet:
            errorMessage = "❌ No Internet Connection\n\nPlease check your network connection."
        case .unsupportedURL:
            errorMessage = "❌ Invalid URL Format\n\nPlease enter a valid URL.\n\nExample: http://192.168.1.5:3000"
        default:
            errorMessage = "❌ Network Error (\(urlError.code.rawValue))\n\n\(urlError.localizedDescription)"
        }
    }

    private func handleUnexpectedError(_ error: Error, apiClient: APIClient, oldURL: String) {
        apiClient.baseURL = oldURL
        try? KeychainService.delete(key: "api_key")
        errorMessage = "❌ Unexpected Error\n\n\(error.localizedDescription)"
    }
}
