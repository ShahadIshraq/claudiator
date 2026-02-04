struct AppNotification: Codable, Identifiable, Hashable {
    var id: String { notificationId }
    let notificationId: String  // maps from "id" in JSON
    let sessionId: String
    let deviceId: String
    let title: String
    let body: String
    let notificationType: String
    let payloadJson: String
    let createdAt: String
    let acknowledged: Bool

    enum CodingKeys: String, CodingKey {
        case notificationId = "id"
        case sessionId, deviceId, title, body
        case notificationType, payloadJson, createdAt, acknowledged
    }
}
