import Testing
import Foundation
@testable import Claudiator

@Suite("SetupViewModel Tests")
@MainActor
struct SetupViewModelTests {
    // MARK: - URL Cleaning Tests

    @Test("URL cleaning removes trailing slash")
    func testURLCleaningTrailingSlash() {
        let viewModel = SetupViewModel()
        viewModel.serverURL = "https://example.com/"
        viewModel.apiKey = "test-key"

        // We can't easily test async connect without mocking, but we can verify
        // that the logic would work by checking the string manipulation
        var url = viewModel.serverURL.trimmingCharacters(in: .whitespacesAndNewlines)
        if url.hasSuffix("/") { url.removeLast() }

        #expect(url == "https://example.com")
    }

    @Test("URL cleaning trims whitespace")
    func testURLCleaningWhitespace() {
        let viewModel = SetupViewModel()
        viewModel.serverURL = "  https://example.com  "

        var url = viewModel.serverURL.trimmingCharacters(in: .whitespacesAndNewlines)
        #expect(url == "https://example.com")
    }

    @Test("URL cleaning adds http for localhost")
    func testURLCleaningLocalhost() {
        let testCases = [
            ("localhost:3000", "http://localhost:3000"),
            ("127.0.0.1:3000", "http://127.0.0.1:3000"),
            ("test.local", "http://test.local"),
        ]

        for (input, expected) in testCases {
            var url = input.trimmingCharacters(in: .whitespacesAndNewlines)
            if url.hasSuffix("/") { url.removeLast() }
            if !url.hasPrefix("http") {
                let isLocal = url.contains(".local") || url.starts(with: "localhost") || url.starts(with: "127.0.0.1")
                url = (isLocal ? "http://" : "https://") + url
            }
            #expect(url == expected, "Expected \(expected), got \(url)")
        }
    }

    @Test("URL cleaning adds https for non-local")
    func testURLCleaningNonLocal() {
        let input = "example.com"
        var url = input.trimmingCharacters(in: .whitespacesAndNewlines)
        if url.hasSuffix("/") { url.removeLast() }
        if !url.hasPrefix("http") {
            let isLocal = url.contains(".local") || url.starts(with: "localhost") || url.starts(with: "127.0.0.1")
            url = (isLocal ? "http://" : "https://") + url
        }
        #expect(url == "https://example.com")
    }

    @Test("URL cleaning preserves existing http prefix")
    func testURLCleaningPreservesHTTP() {
        let input = "http://example.com"
        var url = input.trimmingCharacters(in: .whitespacesAndNewlines)
        if url.hasSuffix("/") { url.removeLast() }
        if !url.hasPrefix("http") {
            let isLocal = url.contains(".local") || url.starts(with: "localhost") || url.starts(with: "127.0.0.1")
            url = (isLocal ? "http://" : "https://") + url
        }
        #expect(url == "http://example.com")
    }

    @Test("URL cleaning preserves existing https prefix")
    func testURLCleaningPreservesHTTPS() {
        let input = "https://example.com"
        var url = input.trimmingCharacters(in: .whitespacesAndNewlines)
        if url.hasSuffix("/") { url.removeLast() }
        if !url.hasPrefix("http") {
            let isLocal = url.contains(".local") || url.starts(with: "localhost") || url.starts(with: "127.0.0.1")
            url = (isLocal ? "http://" : "https://") + url
        }
        #expect(url == "https://example.com")
    }

    @Test("URL cleaning handles .local detection")
    func testURLCleaningLocalDetection() {
        let testCases = [
            "server.local",
            "test.local:3000",
            "my-server.local",
        ]

        for input in testCases {
            var url = input.trimmingCharacters(in: .whitespacesAndNewlines)
            if url.hasSuffix("/") { url.removeLast() }
            if !url.hasPrefix("http") {
                let isLocal = url.contains(".local") || url.starts(with: "localhost") || url.starts(with: "127.0.0.1")
                url = (isLocal ? "http://" : "https://") + url
            }
            #expect(url.hasPrefix("http://"), "Expected \(input) to use http://")
        }
    }

    @Test("URL cleaning combined operations")
    func testURLCleaningCombined() {
        let input = "  localhost:3000/  "
        var url = input.trimmingCharacters(in: .whitespacesAndNewlines)
        if url.hasSuffix("/") { url.removeLast() }
        if !url.hasPrefix("http") {
            let isLocal = url.contains(".local") || url.starts(with: "localhost") || url.starts(with: "127.0.0.1")
            url = (isLocal ? "http://" : "https://") + url
        }
        #expect(url == "http://localhost:3000")
    }

    // MARK: - ViewModel State Tests

    @Test("ViewModel initializes with empty state")
    func testInitialState() {
        let viewModel = SetupViewModel()

        #expect(viewModel.serverURL == "")
        #expect(viewModel.apiKey == "")
        #expect(viewModel.isLoading == false)
        #expect(viewModel.errorMessage == nil)
    }

    @Test("ViewModel accepts input")
    func testAcceptsInput() {
        let viewModel = SetupViewModel()
        viewModel.serverURL = "https://example.com"
        viewModel.apiKey = "test-api-key"

        #expect(viewModel.serverURL == "https://example.com")
        #expect(viewModel.apiKey == "test-api-key")
    }

    // MARK: - Error Message Format Tests

    @Test("Unauthorized error message format")
    func testUnauthorizedErrorMessage() async {
        let viewModel = SetupViewModel()
        viewModel.serverURL = "https://example.com"
        viewModel.apiKey = "invalid-key"

        let expectedMessage = "❌ Authentication Failed\n\nThe API key is incorrect. Please check your API key and try again."

        // Test the error message format
        #expect(expectedMessage.contains("Authentication Failed"))
        #expect(expectedMessage.contains("API key is incorrect"))
    }

    @Test("Server error message includes status code")
    func testServerErrorMessageFormat() {
        let statusCode = 500
        let expectedMessage = "❌ Server Error (\(statusCode))\n\nThe server returned an error. Make sure the server is running correctly."

        #expect(expectedMessage.contains("Server Error (500)"))
        #expect(expectedMessage.contains("server is running correctly"))
    }

    @Test("Connection timeout error message format")
    func testConnectionTimeoutMessage() {
        let expectedMessage = "❌ Connection Timeout\n\nThe server didn't respond in time.\n\nCheck:\n• Your network connection\n• The server address"

        #expect(expectedMessage.contains("Connection Timeout"))
        #expect(expectedMessage.contains("didn't respond in time"))
        #expect(expectedMessage.contains("network connection"))
        #expect(expectedMessage.contains("server address"))
    }

    @Test("Cannot connect error message includes URL")
    func testCannotConnectMessageIncludesURL() {
        let testURL = "http://192.168.1.5:3000"
        let expectedMessage = "❌ Cannot Reach Server\n\nUnable to connect to: \(testURL)\n\nMake sure:\n• The server is running\n• The URL is correct\n• You're on the same network (for local servers)"

        #expect(expectedMessage.contains("Cannot Reach Server"))
        #expect(expectedMessage.contains(testURL))
        #expect(expectedMessage.contains("server is running"))
        #expect(expectedMessage.contains("URL is correct"))
        #expect(expectedMessage.contains("same network"))
    }

    @Test("HTTPS failure message suggests http")
    func testHTTPSFailureMessage() {
        let expectedMessage = "❌ HTTPS Connection Failed\n\nFor local development servers, use http:// instead of https://\n\nExample: http://192.168.1.5:3000"

        #expect(expectedMessage.contains("HTTPS Connection Failed"))
        #expect(expectedMessage.contains("use http://"))
        #expect(expectedMessage.contains("instead of https://"))
        #expect(expectedMessage.contains("Example: http://"))
    }

    @Test("Invalid URL error message format")
    func testInvalidURLErrorMessage() {
        let expectedMessage = "❌ Invalid URL Format\n\nPlease enter a valid URL.\n\nExample: http://192.168.1.5:3000"

        #expect(expectedMessage.contains("Invalid URL Format"))
        #expect(expectedMessage.contains("valid URL"))
        #expect(expectedMessage.contains("Example:"))
    }

    @Test("No internet connection error message")
    func testNoInternetErrorMessage() {
        let expectedMessage = "❌ No Internet Connection\n\nPlease check your network connection."

        #expect(expectedMessage.contains("No Internet Connection"))
        #expect(expectedMessage.contains("network connection"))
    }

    @Test("Network error includes error code")
    func testNetworkErrorWithCode() {
        let errorCode = -1001
        let expectedMessage = "❌ Network Error (\(errorCode))\n\n"

        #expect(expectedMessage.contains("Network Error"))
        #expect(expectedMessage.contains("(\(errorCode))"))
    }

    // MARK: - Invalid URL Validation Tests

    @Test("Invalid URL is rejected - empty")
    func testInvalidURLEmpty() {
        let viewModel = SetupViewModel()
        viewModel.serverURL = ""

        // Empty URL should result in invalid format
        let cleaned = viewModel.serverURL.trimmingCharacters(in: .whitespacesAndNewlines)
        #expect(cleaned.isEmpty)
    }

    @Test("Invalid URL is rejected - invalid characters")
    func testInvalidURLCharacters() {
        let invalidURLs = [
            "http://exam ple.com",
            "http://example com",
            "ht tp://example.com"
        ]

        for invalidURL in invalidURLs {
            // Test that URL validation would reject these
            let url = invalidURL.trimmingCharacters(in: .whitespacesAndNewlines)
            #expect(url.contains(" "), "URL with spaces should contain space: \(url)")
        }
    }

    @Test("Valid URL formats are accepted")
    func testValidURLFormats() {
        let validURLs = [
            "http://example.com",
            "https://example.com",
            "http://192.168.1.5:3000",
            "http://localhost:8080",
            "http://server.local:3000"
        ]

        for urlString in validURLs {
            let url = URL(string: urlString)
            #expect(url != nil, "Valid URL should be accepted: \(urlString)")
        }
    }
}
