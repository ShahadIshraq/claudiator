import Foundation

struct Event: Codable, Identifiable {
    let id: Int
    let hookEventName: String
    let timestamp: String
    let toolName: String?
    let notificationType: String?
    let message: String?
}
