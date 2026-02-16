import Foundation
import Observation

enum APIError: LocalizedError {
    case notConfigured
    case invalidURL
    case unauthorized
    case serverError(Int)
    case networkError(Error)
    case decodingError(Error)

    var errorDescription: String? {
        switch self {
        case .notConfigured: "Server not configured"
        case .invalidURL: "Invalid server URL"
        case .unauthorized: "Invalid API key"
        case let .serverError(code): "Server error (\(code))"
        case let .networkError(error): error.localizedDescription
        case let .decodingError(error): "Data error: \(error.localizedDescription)"
        }
    }
}

@MainActor
@Observable
class APIClient {
    var baseURL: String {
        didSet { UserDefaults.standard.set(baseURL, forKey: "server_url") }
    }

    var isConfigured: Bool {
        !baseURL.isEmpty && (try? KeychainService.load(key: "api_key")) != nil
    }

    private var apiKey: String? {
        try? KeychainService.load(key: "api_key")
    }

    init() {
        self.baseURL = UserDefaults.standard.string(forKey: "server_url") ?? ""
    }

    private static let decoder: JSONDecoder = {
        let d = JSONDecoder()
        d.keyDecodingStrategy = .convertFromSnakeCase
        return d
    }()

    private lazy var debugSession: URLSession = {
        let config = URLSessionConfiguration.default
        return URLSession(configuration: config, delegate: TLSDebugDelegate(), delegateQueue: nil)
    }()

    private class TLSDebugDelegate: NSObject, URLSessionDelegate {
        // swiftlint:disable:next cyclomatic_complexity
        func urlSession(
            _ session: URLSession,
            didReceive challenge: URLAuthenticationChallenge,
            completionHandler: @escaping (URLSession.AuthChallengeDisposition, URLCredential?) -> Void
        ) {
            print("[TLS Debug] Received authentication challenge")
            print("[TLS Debug] Protection space: \(challenge.protectionSpace.host):\(challenge.protectionSpace.port)")
            print("[TLS Debug] Authentication method: \(challenge.protectionSpace.authenticationMethod)")

            if challenge.protectionSpace.authenticationMethod == NSURLAuthenticationMethodServerTrust,
               let serverTrust = challenge.protectionSpace.serverTrust {
                print("[TLS Debug] Server trust received, examining certificate chain")

                let certCount = SecTrustGetCertificateCount(serverTrust)
                print("[TLS Debug] Certificate chain count: \(certCount)")

                for index in 0 ..< certCount {
                    if let cert = SecTrustGetCertificateAtIndex(serverTrust, index) {
                        print("[TLS Debug] --- Certificate \(index) ---")

                        if let subjectSummary = SecCertificateCopySubjectSummary(cert) as String? {
                            print("[TLS Debug] Subject: \(subjectSummary)")
                        }

                        let certData = SecCertificateCopyData(cert) as Data
                        print("[TLS Debug] DER data length: \(certData.count) bytes")

                        if let certDict = SecCertificateCopyValues(cert, nil, nil) as? [String: Any] {
                            print("[TLS Debug] Certificate details available")

                            for (key, value) in certDict {
                                if key.contains("SubjectAltName") || key.contains("Subject Alternative Name") {
                                    print("[TLS Debug] Found SAN entry: \(key) = \(value)")
                                }
                            }

                            if let properties = certDict as CFDictionary?,
                               let sanDict = CFDictionaryGetValue(properties, kSecOIDSubjectAltName as CFString) {
                                print("[TLS Debug] SAN data: \(sanDict)")
                            }
                        }

                        if #available(iOS 15.0, *) {
                            let keys = [kSecOIDSubjectAltName, kSecOIDX509V1SubjectName] as CFArray
                            if let values = SecCertificateCopyValues(cert, keys, nil) as? [String: Any] {
                                for (key, value) in values {
                                    print("[TLS Debug] \(key): \(value)")
                                }
                            }
                        }
                    }
                }
            }

            completionHandler(.performDefaultHandling, nil)
        }
    }

    private func request(_ path: String, method: String = "GET", body: Data? = nil) async throws -> Data {
        guard !baseURL.isEmpty, let key = apiKey else { throw APIError.notConfigured }

        // Construct URL by appending path as string (handles query parameters correctly)
        let urlString = baseURL.hasSuffix("/") ? baseURL + path.dropFirst() : baseURL + path
        guard let url = URL(string: urlString) else { throw APIError.invalidURL }

        var req = URLRequest(url: url)
        req.httpMethod = method
        req.setValue("Bearer \(key)", forHTTPHeaderField: "Authorization")
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        if let body { req.httpBody = body }

        print("[API Request] \(method) \(url.absoluteString)")

        let (data, response): (Data, URLResponse)
        do {
            (data, response) = try await debugSession.data(for: req)
        } catch let urlError as URLError {
            print("[API Error] URLError occurred")
            print("[API Error] Error code (rawValue): \(urlError.code.rawValue)")
            print("[API Error] Error code (errorCode): \(urlError.errorCode)")
            print("[API Error] Localized description: \(urlError.localizedDescription)")
            if let failureURL = urlError.failureURLString {
                print("[API Error] Failure URL: \(failureURL)")
            }
            if let underlyingError = urlError.underlyingError {
                print("[API Error] Underlying error: \(underlyingError)")
            }
            throw urlError
        } catch {
            print("[API Error] Non-URLError: \(error)")
            throw APIError.networkError(error)
        }

        guard let http = response as? HTTPURLResponse else { throw APIError.networkError(URLError(.badServerResponse)) }
        if http.statusCode == 401 { throw APIError.unauthorized }
        guard (200 ... 299).contains(http.statusCode) else { throw APIError.serverError(http.statusCode) }
        return data
    }

    func ping() async throws -> (dataVersion: UInt64, notificationVersion: UInt64) {
        let data = try await request("/api/v1/ping")
        struct PingResponse: Decodable {
            let status: String
            let serverVersion: String?
            let dataVersion: UInt64?
            let notificationVersion: UInt64?
        }
        let response = try Self.decoder.decode(PingResponse.self, from: data)
        return (response.dataVersion ?? 0, response.notificationVersion ?? 0)
    }

    func fetchDevices() async throws -> [Device] {
        let data = try await request("/api/v1/devices")
        struct Wrapper: Decodable { let devices: [Device] }
        return try Self.decoder.decode(Wrapper.self, from: data).devices
    }

    func fetchSessions(deviceId: String, status: String? = nil, limit: Int? = nil) async throws -> [Session] {
        var path = "/api/v1/devices/\(deviceId)/sessions"
        var params: [String] = []
        if let status { params.append("status=\(status)") }
        if let limit { params.append("limit=\(limit)") }
        if !params.isEmpty { path += "?" + params.joined(separator: "&") }
        let data = try await request(path)
        struct Wrapper: Decodable { let sessions: [Session] }
        return try Self.decoder.decode(Wrapper.self, from: data).sessions
    }

    func fetchAllSessions(status: String? = nil, limit: Int? = nil) async throws -> [Session] {
        var path = "/api/v1/sessions"
        var params: [String] = []
        if let status { params.append("status=\(status)") }
        if let limit { params.append("limit=\(limit)") }
        if !params.isEmpty { path += "?" + params.joined(separator: "&") }
        let data = try await request(path)
        struct Wrapper: Decodable { let sessions: [Session] }
        return try Self.decoder.decode(Wrapper.self, from: data).sessions
    }

    func fetchEvents(sessionId: String, limit: Int? = nil) async throws -> [Event] {
        var path = "/api/v1/sessions/\(sessionId)/events"
        if let limit { path += "?limit=\(limit)" }
        let data = try await request(path)
        struct Wrapper: Decodable { let events: [Event] }
        return try Self.decoder.decode(Wrapper.self, from: data).events
    }

    func registerPushToken(platform: String, token: String, sandbox: Bool) async throws {
        struct PushRegisterBody: Encodable {
            let platform: String
            let pushToken: String
            let sandbox: Bool

            enum CodingKeys: String, CodingKey {
                case platform
                case pushToken = "push_token"
                case sandbox
            }
        }
        let body = try JSONEncoder().encode(PushRegisterBody(platform: platform, pushToken: token, sandbox: sandbox))
        _ = try await request("/api/v1/push/register", method: "POST", body: body)
    }

    func fetchNotifications(after: String? = nil, limit: Int? = nil) async throws -> [AppNotification] {
        var path = "/api/v1/notifications"
        var params: [String] = []
        if let after { params.append("after=\(after)") }
        if let limit { params.append("limit=\(limit)") }
        if !params.isEmpty { path += "?" + params.joined(separator: "&") }
        let data = try await request(path)
        struct Wrapper: Decodable { let notifications: [AppNotification] }
        return try Self.decoder.decode(Wrapper.self, from: data).notifications
    }

    func acknowledgeNotifications(ids: [String]) async throws {
        let body = try JSONEncoder().encode(["notification_ids": ids])
        _ = try await request("/api/v1/notifications/ack", method: "POST", body: body)
    }

    func configure(url: String, apiKey: String) throws {
        baseURL = url
        try KeychainService.save(key: "api_key", value: apiKey)
    }

    func disconnect() {
        baseURL = ""
        try? KeychainService.delete(key: "api_key")
    }
}
