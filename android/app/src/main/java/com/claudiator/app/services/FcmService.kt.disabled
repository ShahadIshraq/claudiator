package com.claudiator.app.services

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationCompat
import com.claudiator.app.MainActivity
import com.claudiator.app.R
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

class FcmService : FirebaseMessagingService() {

    private val serviceScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    override fun onNewToken(token: String) {
        getSharedPreferences("claudiator_fcm", Context.MODE_PRIVATE)
            .edit()
            .putString("fcm_token", token)
            .apply()

        val app = application as? com.claudiator.app.ClaudiatorApp ?: return
        if (app.secureStorage.isConfigured) {
            serviceScope.launch {
                try {
                    app.apiClient.registerPushToken(token)
                } catch (_: Exception) {
                    // Will retry on next app launch
                }
            }
        }
    }

    override fun onMessageReceived(message: RemoteMessage) {
        val data = message.data
        val notificationId = data["notification_id"] ?: return
        val sessionId = data["session_id"] ?: return
        val title = data["title"] ?: "Claudiator"
        val body = data["body"] ?: ""

        val app = application as? com.claudiator.app.ClaudiatorApp ?: return

        app.notificationManager.markReceivedViaPush(notificationId)

        showNotification(notificationId, sessionId, title, body)
    }

    private fun showNotification(notificationId: String, sessionId: String, title: String, body: String) {
        val notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

        val channel = NotificationChannel(
            CHANNEL_ID,
            getString(R.string.notification_channel_name),
            NotificationManager.IMPORTANCE_HIGH,
        ).apply {
            description = getString(R.string.notification_channel_description)
            enableVibration(true)
        }
        notificationManager.createNotificationChannel(channel)

        val intent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP
            putExtra("session_id", sessionId)
            putExtra("notification_id", notificationId)
        }
        val pendingIntent = PendingIntent.getActivity(
            this, notificationId.hashCode(), intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )

        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_launcher_foreground)
            .setContentTitle(title)
            .setContentText(body)
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setAutoCancel(true)
            .setContentIntent(pendingIntent)
            .build()

        notificationManager.notify(notificationId.hashCode(), notification)
    }

    companion object {
        const val CHANNEL_ID = "claudiator_sessions"

        fun getStoredToken(context: Context): String? =
            context.getSharedPreferences("claudiator_fcm", Context.MODE_PRIVATE)
                .getString("fcm_token", null)
    }
}
