import Foundation

struct Device: Codable, Identifiable {
    var id: String { deviceId }
    let deviceId: String
    let deviceName: String
    let platform: String
    let firstSeen: String
    let lastSeen: String
    let activeSessions: Int
}
