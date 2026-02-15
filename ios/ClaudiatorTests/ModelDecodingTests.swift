import Testing
import Foundation
@testable import Claudiator

@Suite("Model Decoding Tests")
struct ModelDecodingTests {
    // MARK: - Session Decoding

    @Test("Decode Session with all fields")
    func testDecodeSessionComplete() throws {
        let json = """
        {
            "session_id": "sess_123",
            "device_id": "dev_456",
            "started_at": "2024-01-15T10:30:00Z",
            "last_event": "2024-01-15T11:00:00Z",
            "status": "active",
            "cwd": "/Users/test/project",
            "title": "Test Session",
            "device_name": "MacBook Pro",
            "platform": "darwin"
        }
        """

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let data = json.data(using: .utf8)!
        let session = try decoder.decode(Session.self, from: data)

        #expect(session.sessionId == "sess_123")
        #expect(session.deviceId == "dev_456")
        #expect(session.startedAt == "2024-01-15T10:30:00Z")
        #expect(session.lastEvent == "2024-01-15T11:00:00Z")
        #expect(session.status == "active")
        #expect(session.cwd == "/Users/test/project")
        #expect(session.title == "Test Session")
        #expect(session.deviceName == "MacBook Pro")
        #expect(session.platform == "darwin")
    }

    @Test("Decode Session with missing optional fields")
    func testDecodeSessionMinimal() throws {
        let json = """
        {
            "session_id": "sess_123",
            "device_id": "dev_456",
            "started_at": "2024-01-15T10:30:00Z",
            "last_event": "2024-01-15T11:00:00Z",
            "status": "idle"
        }
        """

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let data = json.data(using: .utf8)!
        let session = try decoder.decode(Session.self, from: data)

        #expect(session.sessionId == "sess_123")
        #expect(session.cwd == nil)
        #expect(session.title == nil)
        #expect(session.deviceName == nil)
        #expect(session.platform == nil)
    }

    @Test("Decode Session with unknown status")
    func testDecodeSessionUnknownStatus() throws {
        let json = """
        {
            "session_id": "sess_123",
            "device_id": "dev_456",
            "started_at": "2024-01-15T10:30:00Z",
            "last_event": "2024-01-15T11:00:00Z",
            "status": "unknown_status"
        }
        """

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let data = json.data(using: .utf8)!
        let session = try decoder.decode(Session.self, from: data)

        #expect(session.status == "unknown_status")
    }

    // MARK: - Device Decoding

    @Test("Decode Device with all fields")
    func testDecodeDevice() throws {
        let json = """
        {
            "device_id": "dev_789",
            "device_name": "Linux Server",
            "platform": "linux",
            "first_seen": "2024-01-10T08:00:00Z",
            "last_seen": "2024-01-15T12:00:00Z",
            "active_sessions": 3
        }
        """

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let data = json.data(using: .utf8)!
        let device = try decoder.decode(Device.self, from: data)

        #expect(device.deviceId == "dev_789")
        #expect(device.deviceName == "Linux Server")
        #expect(device.platform == "linux")
        #expect(device.firstSeen == "2024-01-10T08:00:00Z")
        #expect(device.lastSeen == "2024-01-15T12:00:00Z")
        #expect(device.activeSessions == 3)
    }

    // MARK: - Event Decoding

    @Test("Decode Event with all fields")
    func testDecodeEventComplete() throws {
        let json = """
        {
            "id": 42,
            "hook_event_name": "SessionStart",
            "timestamp": "2024-01-15T10:30:00Z",
            "tool_name": "Read",
            "notification_type": "info",
            "message": "Session started"
        }
        """

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let data = json.data(using: .utf8)!
        let event = try decoder.decode(Event.self, from: data)

        #expect(event.id == 42)
        #expect(event.hookEventName == "SessionStart")
        #expect(event.timestamp == "2024-01-15T10:30:00Z")
        #expect(event.toolName == "Read")
        #expect(event.notificationType == "info")
        #expect(event.message == "Session started")
    }

    @Test("Decode Event with missing optional fields")
    func testDecodeEventMinimal() throws {
        let json = """
        {
            "id": 99,
            "hook_event_name": "Stop",
            "timestamp": "2024-01-15T12:00:00Z"
        }
        """

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let data = json.data(using: .utf8)!
        let event = try decoder.decode(Event.self, from: data)

        #expect(event.id == 99)
        #expect(event.hookEventName == "Stop")
        #expect(event.toolName == nil)
        #expect(event.notificationType == nil)
        #expect(event.message == nil)
    }

    // MARK: - AppNotification Decoding

    @Test("Decode AppNotification with all fields")
    func testDecodeAppNotificationComplete() throws {
        let json = """
        {
            "id": "notif_123",
            "session_id": "sess_456",
            "device_id": "dev_789",
            "title": "New Event",
            "body": "Tool execution completed",
            "notification_type": "info",
            "payload_json": "{\\"key\\": \\"value\\"}",
            "created_at": "2024-01-15T14:00:00Z",
            "acknowledged": false
        }
        """

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let data = json.data(using: .utf8)!
        let notification = try decoder.decode(AppNotification.self, from: data)

        #expect(notification.notificationId == "notif_123")
        #expect(notification.sessionId == "sess_456")
        #expect(notification.deviceId == "dev_789")
        #expect(notification.title == "New Event")
        #expect(notification.body == "Tool execution completed")
        #expect(notification.notificationType == "info")
        #expect(notification.payloadJson == #"{"key": "value"}"#)
        #expect(notification.createdAt == "2024-01-15T14:00:00Z")
        #expect(notification.acknowledged == false)
    }

    @Test("Decode AppNotification with missing optional fields")
    func testDecodeAppNotificationMinimal() throws {
        let json = """
        {
            "id": "notif_456",
            "session_id": "sess_789",
            "device_id": "dev_123",
            "title": "Test",
            "body": "Test body",
            "notification_type": "warning",
            "created_at": "2024-01-15T15:00:00Z"
        }
        """

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let data = json.data(using: .utf8)!
        let notification = try decoder.decode(AppNotification.self, from: data)

        #expect(notification.notificationId == "notif_456")
        #expect(notification.payloadJson == nil)
        #expect(notification.acknowledged == nil)
    }
}
