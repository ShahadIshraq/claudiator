package com.claudiator.app.services

import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class VersionMonitor(
    private val apiClient: ApiClient,
    private val notificationManager: AppNotificationManager,
) {
    private val _dataVersion = MutableStateFlow(0L)
    val dataVersion: StateFlow<Long> = _dataVersion.asStateFlow()

    private var notificationVersion = 0L
    private var pollingJob: Job? = null

    fun start(scope: CoroutineScope) {
        if (pollingJob != null) return
        pollingJob = scope.launch {
            while (isActive) {
                try {
                    val ping = apiClient.ping()
                    _dataVersion.value = ping.dataVersion
                    if (ping.notificationVersion != notificationVersion) {
                        notificationVersion = ping.notificationVersion
                        val lastSeen = notificationManager.lastSeenId
                        val notifications = apiClient.fetchNotifications(after = lastSeen)
                        notificationManager.processNotifications(notifications)
                    }
                } catch (_: Exception) {
                    // silently retry next cycle
                }
                delay(10_000)
            }
        }
    }

    fun stop() {
        pollingJob?.cancel()
        pollingJob = null
    }
}
