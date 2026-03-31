package com.claudiator.app

import com.claudiator.app.models.*
import kotlinx.serialization.json.Json
import org.junit.Assert.*
import org.junit.Test

class ModelDecodingTest {

    private val json = Json { ignoreUnknownKeys = true }

    @Test
    fun `decode Device with all fields`() {
        val raw = """
        {
            "device_id": "dev_789",
            "device_name": "Linux Server",
            "platform": "linux",
            "first_seen": "2024-01-10T08:00:00Z",
            "last_seen": "2024-01-15T12:00:00Z",
            "active_sessions": 3
        }
        """
        val device = json.decodeFromString<Device>(raw)
        assertEquals("dev_789", device.deviceId)
        assertEquals("Linux Server", device.deviceName)
        assertEquals("linux", device.platform)
        assertEquals("2024-01-10T08:00:00Z", device.firstSeen)
        assertEquals("2024-01-15T12:00:00Z", device.lastSeen)
        assertEquals(3, device.activeSessions)
    }

    @Test
    fun `decode Session with all fields`() {
        val raw = """
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
        val session = json.decodeFromString<Session>(raw)
        assertEquals("sess_123", session.sessionId)
        assertEquals("dev_456", session.deviceId)
        assertEquals("active", session.status)
        assertEquals("/Users/test/project", session.cwd)
        assertEquals("Test Session", session.title)
        assertEquals("MacBook Pro", session.deviceName)
        assertEquals("darwin", session.platform)
    }

    @Test
    fun `decode Session with missing optional fields`() {
        val raw = """
        {
            "session_id": "sess_123",
            "device_id": "dev_456",
            "started_at": "2024-01-15T10:30:00Z",
            "last_event": "2024-01-15T11:00:00Z",
            "status": "idle"
        }
        """
        val session = json.decodeFromString<Session>(raw)
        assertEquals("sess_123", session.sessionId)
        assertNull(session.cwd)
        assertNull(session.title)
        assertNull(session.deviceName)
        assertNull(session.platform)
    }

    @Test
    fun `decode Session with unknown status`() {
        val raw = """
        {
            "session_id": "sess_123",
            "device_id": "dev_456",
            "started_at": "2024-01-15T10:30:00Z",
            "last_event": "2024-01-15T11:00:00Z",
            "status": "unknown_status"
        }
        """
        val session = json.decodeFromString<Session>(raw)
        assertEquals("unknown_status", session.status)
    }

    @Test
    fun `decode Event with all fields`() {
        val raw = """
        {
            "id": 42,
            "hook_event_name": "SessionStart",
            "timestamp": "2024-01-15T10:30:00Z",
            "tool_name": "Read",
            "notification_type": "info",
            "message": "Session started"
        }
        """
        val event = json.decodeFromString<Event>(raw)
        assertEquals(42, event.id)
        assertEquals("SessionStart", event.hookEventName)
        assertEquals("Read", event.toolName)
        assertEquals("info", event.notificationType)
        assertEquals("Session started", event.message)
    }

    @Test
    fun `decode Event with missing optional fields`() {
        val raw = """
        {
            "id": 99,
            "hook_event_name": "Stop",
            "timestamp": "2024-01-15T12:00:00Z"
        }
        """
        val event = json.decodeFromString<Event>(raw)
        assertEquals(99, event.id)
        assertEquals("Stop", event.hookEventName)
        assertNull(event.toolName)
        assertNull(event.notificationType)
        assertNull(event.message)
    }

    @Test
    fun `decode AppNotification with all fields`() {
        val raw = """
        {
            "id": "notif_123",
            "session_id": "sess_456",
            "device_id": "dev_789",
            "title": "New Event",
            "body": "Tool execution completed",
            "notification_type": "info",
            "payload_json": "{\"key\": \"value\"}",
            "created_at": "2024-01-15T14:00:00Z",
            "acknowledged": false
        }
        """
        val notif = json.decodeFromString<AppNotification>(raw)
        assertEquals("notif_123", notif.notificationId)
        assertEquals("sess_456", notif.sessionId)
        assertEquals("dev_789", notif.deviceId)
        assertEquals("New Event", notif.title)
        assertEquals("Tool execution completed", notif.body)
        assertEquals("info", notif.notificationType)
        assertEquals("""{"key": "value"}""", notif.payloadJson)
        assertEquals("2024-01-15T14:00:00Z", notif.createdAt)
        assertEquals(false, notif.acknowledged)
    }

    @Test
    fun `decode AppNotification with missing optional fields`() {
        val raw = """
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
        val notif = json.decodeFromString<AppNotification>(raw)
        assertEquals("notif_456", notif.notificationId)
        assertNull(notif.payloadJson)
        assertNull(notif.acknowledged)
    }

    @Test
    fun `decode PingResponse`() {
        val raw = """
        {
            "status": "ok",
            "server_version": "0.4.3",
            "data_version": 42,
            "notification_version": 7
        }
        """
        val ping = json.decodeFromString<PingResponse>(raw)
        assertEquals("ok", ping.status)
        assertEquals(42L, ping.dataVersion)
        assertEquals(7L, ping.notificationVersion)
    }

    @Test
    fun `decode SessionListPage`() {
        val raw = """
        {
            "sessions": [
                {
                    "session_id": "s1",
                    "device_id": "d1",
                    "started_at": "2024-01-15T10:00:00Z",
                    "last_event": "2024-01-15T11:00:00Z",
                    "status": "active"
                }
            ],
            "has_more": true,
            "next_offset": 50
        }
        """
        val page = json.decodeFromString<SessionListPage>(raw)
        assertEquals(1, page.sessions.size)
        assertTrue(page.hasMore)
        assertEquals(50, page.nextOffset)
    }
}
