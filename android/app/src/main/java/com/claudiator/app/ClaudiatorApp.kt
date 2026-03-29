package com.claudiator.app

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager
import com.claudiator.app.services.*
import com.claudiator.app.ui.theme.ThemeManager

class ClaudiatorApp : Application() {

    lateinit var secureStorage: SecureStorage
    lateinit var apiClient: ApiClient
    lateinit var notificationManager: AppNotificationManager
    lateinit var versionMonitor: VersionMonitor
    lateinit var themeManager: ThemeManager

    override fun onCreate() {
        super.onCreate()
        secureStorage = SecureStorage(this)
        apiClient = ApiClient(secureStorage)
        notificationManager = AppNotificationManager()
        versionMonitor = VersionMonitor(apiClient, notificationManager)
        themeManager = ThemeManager(this)

        createNotificationChannel()
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            "claudiator_sessions",
            getString(R.string.notification_channel_name),
            NotificationManager.IMPORTANCE_HIGH,
        ).apply {
            description = getString(R.string.notification_channel_description)
            enableVibration(true)
        }
        val manager = getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(channel)
    }
}
