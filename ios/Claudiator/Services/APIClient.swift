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
        case .notConfigured: return "Server not configured"
        case .invalidURL: return "Invalid server URL"
        case .unauthorized: return "Invalid API key"
        case .serverError(let code): return "Server error (\(code))"
        case .networkError(let error): return error.localizedDescription
        case .decodingError(let error): return "Data error: \(error.localizedDescription)"
        }
    }
}

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

        let (data, response): (Data, URLResponse)
        do {
            (data, response) = try await URLSession.shared.data(for: req)
        } catch let urlError as URLError {
            throw urlError
        } catch {
            throw APIError.networkError(error)
        }

        guard let http = response as? HTTPURLResponse else { throw APIError.networkError(URLError(.badServerResponse)) }
        if http.statusCode == 401 { throw APIError.unauthorized }
        guard (200...299).contains(http.statusCode) else { throw APIError.serverError(http.statusCode) }
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

    func fetchNotifications(since: String? = nil, limit: Int? = nil) async throws -> [AppNotification] {
        var path = "/api/v1/notifications"
        var params: [String] = []
        if let since { params.append("since=\(since)") }
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
