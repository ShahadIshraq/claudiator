package com.claudiator.app.services

import com.claudiator.app.models.AppNotification
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

data class NotificationState(
    val notifications: List<AppNotification> = emptyList(),
    val unreadCount: Int = 0,
    val unreadSessionIds: Set<String> = emptySet(),
    val readIds: Set<String> = emptySet(),
)

class AppNotificationManager {

    private val _state = MutableStateFlow(NotificationState())
    val state: StateFlow<NotificationState> = _state.asStateFlow()

    private val pushReceivedIds = mutableMapOf<String, Long>()
    private val pushRetentionMs = 10 * 60 * 1000L

    private val maxNotifications = 100
    private val maxReadIds = 500

    var lastSeenId: String? = null
        private set

    fun processNotifications(incoming: List<AppNotification>) {
        if (incoming.isEmpty()) return

        incoming.lastOrNull()?.let { lastSeenId = it.createdAt }

        _state.update { current ->
            val existingIds = current.notifications.map { it.notificationId }.toSet()
            val newNotifs = incoming.filter { it.notificationId !in existingIds }

            val allNotifs = (newNotifs + current.notifications).take(maxNotifications)

            val serverAckedIds = incoming
                .filter { it.acknowledged == true }
                .map { it.notificationId }
                .toSet()
            val readIds = (current.readIds + serverAckedIds).let { ids ->
                if (ids.size > maxReadIds) ids.sorted().takeLast(maxReadIds).toSet() else ids
            }

            val unread = allNotifs.filter { it.notificationId !in readIds }

            current.copy(
                notifications = allNotifs,
                readIds = readIds,
                unreadCount = unread.size,
                unreadSessionIds = unread.map { it.sessionId }.toSet(),
            )
        }
    }

    fun markSessionRead(sessionId: String): List<String> {
        var markedIds = emptyList<String>()
        _state.update { current ->
            val toMark = current.notifications
                .filter { it.sessionId == sessionId && it.notificationId !in current.readIds }
                .map { it.notificationId }
            if (toMark.isEmpty()) return@update current

            markedIds = toMark
            val readIds = trimReadIds(current.readIds + toMark)
            val unread = current.notifications.filter { it.notificationId !in readIds }

            current.copy(
                readIds = readIds,
                unreadCount = unread.size,
                unreadSessionIds = unread.map { it.sessionId }.toSet(),
            )
        }
        return markedIds
    }

    fun markNotificationRead(notificationId: String): String? {
        var marked: String? = null
        _state.update { current ->
            if (notificationId in current.readIds ||
                current.notifications.none { it.notificationId == notificationId }
            ) {
                return@update current
            }

            marked = notificationId
            val readIds = trimReadIds(current.readIds + notificationId)
            val unread = current.notifications.filter { it.notificationId !in readIds }

            current.copy(
                readIds = readIds,
                unreadCount = unread.size,
                unreadSessionIds = unread.map { it.sessionId }.toSet(),
            )
        }
        return marked
    }

    fun markAllRead(): List<String> {
        var markedIds = emptyList<String>()
        _state.update { current ->
            val unreadIds = current.notifications
                .filter { it.notificationId !in current.readIds }
                .map { it.notificationId }
            if (unreadIds.isEmpty()) return@update current

            markedIds = unreadIds
            val readIds = trimReadIds(current.readIds + unreadIds)

            current.copy(
                readIds = readIds,
                unreadCount = 0,
                unreadSessionIds = emptySet(),
            )
        }
        return markedIds
    }

    fun markReceivedViaPush(notificationId: String) {
        cleanupOldPushIds()
        pushReceivedIds[notificationId] = System.currentTimeMillis()
    }

    fun isPushReceived(notificationId: String): Boolean {
        cleanupOldPushIds()
        return notificationId in pushReceivedIds
    }

    private fun cleanupOldPushIds() {
        val cutoff = System.currentTimeMillis() - pushRetentionMs
        pushReceivedIds.entries.removeAll { it.value < cutoff }
    }

    private fun trimReadIds(ids: Set<String>): Set<String> {
        return if (ids.size > maxReadIds) ids.sorted().takeLast(maxReadIds).toSet() else ids
    }
}
