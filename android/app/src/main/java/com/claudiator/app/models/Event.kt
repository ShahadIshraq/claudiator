package com.claudiator.app.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class Event(
    val id: Int,
    @SerialName("hook_event_name") val hookEventName: String,
    val timestamp: String,
    @SerialName("tool_name") val toolName: String? = null,
    @SerialName("notification_type") val notificationType: String? = null,
    val message: String? = null,
)
