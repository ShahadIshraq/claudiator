import Foundation

extension Device: Hashable {
    static func == (lhs: Device, rhs: Device) -> Bool { lhs.deviceId == rhs.deviceId }
    func hash(into hasher: inout Hasher) { hasher.combine(deviceId) }
}

extension Session: Hashable {
    static func == (lhs: Session, rhs: Session) -> Bool { lhs.sessionId == rhs.sessionId }
    func hash(into hasher: inout Hasher) { hasher.combine(sessionId) }
}

extension Event: Hashable {
    static func == (lhs: Event, rhs: Event) -> Bool { lhs.id == rhs.id }
    func hash(into hasher: inout Hasher) { hasher.combine(id) }
}
