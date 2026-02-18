import Foundation

enum URLValidator {
    /// Cleans and validates a server URL string.
    /// Returns the cleaned URL string, or nil if invalid.
    static func cleanAndValidate(_ input: String) -> String? {
        var url = input.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !url.isEmpty else { return nil }
        if url.hasSuffix("/") { url.removeLast() }
        if !url.hasPrefix("http") {
            let isLocal = url.contains(".local") || url.starts(with: "localhost") || url.starts(with: "127.0.0.1")
            url = (isLocal ? "http://" : "https://") + url
        }
        guard URL(string: url) != nil else { return nil }
        return url
    }
}
