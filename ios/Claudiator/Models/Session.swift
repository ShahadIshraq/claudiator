import Foundation

struct Session: Codable, Identifiable {
    var id: String { sessionId }
    let sessionId: String
    let deviceId: String
    let startedAt: String
    let lastEvent: String
    let status: String
    let cwd: String?
    let title: String?
    let deviceName: String?
    let platform: String?
}
