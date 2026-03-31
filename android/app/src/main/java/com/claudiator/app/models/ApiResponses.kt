package com.claudiator.app.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class PingResponse(
    val status: String,
    @SerialName("server_version") val serverVersion: String? = null,
    @SerialName("data_version") val dataVersion: Long = 0,
    @SerialName("notification_version") val notificationVersion: Long = 0,
)

@Serializable
data class DevicesResponse(val devices: List<Device>)

@Serializable
data class SessionsResponse(val sessions: List<Session>)

@Serializable
data class SessionListPage(
    val sessions: List<Session>,
    @SerialName("has_more") val hasMore: Boolean,
    @SerialName("next_offset") val nextOffset: Int,
)

@Serializable
data class EventsResponse(val events: List<Event>)

@Serializable
data class NotificationsResponse(val notifications: List<AppNotification>)
