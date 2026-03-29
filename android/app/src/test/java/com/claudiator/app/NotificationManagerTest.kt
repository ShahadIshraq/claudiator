package com.claudiator.app

import com.claudiator.app.models.AppNotification
import com.claudiator.app.services.AppNotificationManager
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test

class NotificationManagerTest {

    private lateinit var manager: AppNotificationManager

    private fun notif(id: String, sessionId: String = "sess1") = AppNotification(
        notificationId = id,
        sessionId = sessionId,
        deviceId = "dev1",
        title = "Test",
        body = "Test body",
        notificationType = "info",
        createdAt = "2024-01-15T10:00:00Z",
    )

    @Before
    fun setUp() {
        manager = AppNotificationManager()
    }

    @Test
    fun `initializes with empty state`() {
        assertEquals(0, manager.state.value.unreadCount)
        assertTrue(manager.state.value.notifications.isEmpty())
        assertTrue(manager.state.value.unreadSessionIds.isEmpty())
    }

    @Test
    fun `processNotifications adds unread notifications`() {
        manager.processNotifications(listOf(notif("n1"), notif("n2")))
        assertEquals(2, manager.state.value.unreadCount)
        assertEquals(2, manager.state.value.notifications.size)
    }

    @Test
    fun `markSessionRead removes notifications for session`() {
        manager.processNotifications(listOf(
            notif("n1", "sess1"),
            notif("n2", "sess1"),
            notif("n3", "sess2"),
        ))
        val acked = manager.markSessionRead("sess1")
        assertEquals(listOf("n1", "n2"), acked)
        assertEquals(1, manager.state.value.unreadCount)
        assertFalse(manager.state.value.unreadSessionIds.contains("sess1"))
        assertTrue(manager.state.value.unreadSessionIds.contains("sess2"))
    }

    @Test
    fun `markSessionRead handles non-existent session`() {
        manager.processNotifications(listOf(notif("n1", "sess1")))
        val acked = manager.markSessionRead("sess99")
        assertTrue(acked.isEmpty())
        assertEquals(1, manager.state.value.unreadCount)
    }

    @Test
    fun `markNotificationRead removes specific notification`() {
        manager.processNotifications(listOf(notif("n1"), notif("n2")))
        val acked = manager.markNotificationRead("n1")
        assertEquals("n1", acked)
        assertEquals(1, manager.state.value.unreadCount)
    }

    @Test
    fun `markNotificationRead returns null for non-existent`() {
        manager.processNotifications(listOf(notif("n1")))
        assertNull(manager.markNotificationRead("n99"))
        assertEquals(1, manager.state.value.unreadCount)
    }

    @Test
    fun `markAllRead clears all unread`() {
        manager.processNotifications(listOf(notif("n1"), notif("n2"), notif("n3")))
        val acked = manager.markAllRead()
        assertEquals(3, acked.size)
        assertEquals(0, manager.state.value.unreadCount)
        assertTrue(manager.state.value.unreadSessionIds.isEmpty())
    }

    @Test
    fun `markAllRead returns empty when no unread`() {
        assertTrue(manager.markAllRead().isEmpty())
    }

    @Test
    fun `duplicate notifications are not added`() {
        manager.processNotifications(listOf(notif("n1")))
        manager.processNotifications(listOf(notif("n1")))
        assertEquals(1, manager.state.value.notifications.size)
    }

    @Test
    fun `caps at 100 notifications`() {
        val notifs = (1..120).map { notif("n$it") }
        manager.processNotifications(notifs)
        assertEquals(100, manager.state.value.notifications.size)
    }

    @Test
    fun `markReceivedViaPush tracks push-received IDs`() {
        manager.markReceivedViaPush("n1")
        assertTrue(manager.isPushReceived("n1"))
        assertFalse(manager.isPushReceived("n2"))
    }

    @Test
    fun `acknowledged notifications are marked read`() {
        val acked = notif("n1").copy(acknowledged = true)
        manager.processNotifications(listOf(acked, notif("n2")))
        assertEquals(1, manager.state.value.unreadCount)
    }
}
