package com.claudiator.app.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class AppNotification(
    @SerialName("id") val notificationId: String,
    @SerialName("session_id") val sessionId: String,
    @SerialName("device_id") val deviceId: String,
    val title: String,
    val body: String,
    @SerialName("notification_type") val notificationType: String,
    @SerialName("payload_json") val payloadJson: String? = null,
    @SerialName("created_at") val createdAt: String,
    val acknowledged: Boolean? = null,
)
