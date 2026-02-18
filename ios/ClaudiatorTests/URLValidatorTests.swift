import Testing
import Foundation
@testable import Claudiator

@Suite("URLValidator Tests")
struct URLValidatorTests {
    // MARK: - Empty / Whitespace Input

    @Test("Empty input returns nil")
    func emptyInput() {
        #expect(URLValidator.cleanAndValidate("") == nil)
    }

    @Test("Whitespace-only input returns nil")
    func whitespaceOnlyInput() {
        #expect(URLValidator.cleanAndValidate("   ") == nil)
        #expect(URLValidator.cleanAndValidate("\t\n") == nil)
    }

    // MARK: - Trailing Slash Removal

    @Test("Trailing slash is removed")
    func trailingSlashRemoval() {
        #expect(URLValidator.cleanAndValidate("https://example.com/") == "https://example.com")
    }

    @Test("No trailing slash is left unchanged")
    func noTrailingSlash() {
        #expect(URLValidator.cleanAndValidate("https://example.com") == "https://example.com")
    }

    // MARK: - Scheme Inference

    @Test("localhost gets http:// prefix")
    func localhostGetsHTTP() {
        #expect(URLValidator.cleanAndValidate("localhost:3000") == "http://localhost:3000")
    }

    @Test(".local hostname gets http:// prefix")
    func localDomainGetsHTTP() {
        #expect(URLValidator.cleanAndValidate("server.local") == "http://server.local")
        #expect(URLValidator.cleanAndValidate("test.local:3000") == "http://test.local:3000")
    }

    @Test("127.0.0.1 gets http:// prefix")
    func loopbackGetsHTTP() {
        #expect(URLValidator.cleanAndValidate("127.0.0.1:8080") == "http://127.0.0.1:8080")
    }

    @Test("Remote hostname gets https:// prefix")
    func remoteGetsHTTPS() {
        #expect(URLValidator.cleanAndValidate("example.com") == "https://example.com")
        #expect(URLValidator.cleanAndValidate("api.myapp.io") == "https://api.myapp.io")
    }

    // MARK: - Existing Prefix Preservation

    @Test("Existing http:// prefix is preserved")
    func existingHTTPPreserved() {
        #expect(URLValidator.cleanAndValidate("http://example.com") == "http://example.com")
    }

    @Test("Existing https:// prefix is preserved")
    func existingHTTPSPreserved() {
        #expect(URLValidator.cleanAndValidate("https://example.com") == "https://example.com")
    }

    // MARK: - Combined Operations

    @Test("Whitespace trimming and trailing slash removal combined")
    func whitespaceAndTrailingSlash() {
        #expect(URLValidator.cleanAndValidate("  https://example.com/  ") == "https://example.com")
    }

    @Test("Localhost with trailing slash gets cleaned")
    func localhostWithTrailingSlash() {
        #expect(URLValidator.cleanAndValidate("localhost:3000/") == "http://localhost:3000")
    }

    // MARK: - Invalid URL

    @Test("String that produces invalid URL returns nil")
    func invalidURLReturnsNil() {
        // A string with spaces in the middle cannot form a valid URL
        #expect(URLValidator.cleanAndValidate("http://exam ple.com") == nil)
    }
}
