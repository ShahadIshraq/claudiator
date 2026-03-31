package com.claudiator.app.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class Session(
    @SerialName("session_id") val sessionId: String,
    @SerialName("device_id") val deviceId: String,
    @SerialName("started_at") val startedAt: String,
    @SerialName("last_event") val lastEvent: String,
    val status: String,
    val cwd: String? = null,
    val title: String? = null,
    @SerialName("device_name") val deviceName: String? = null,
    val platform: String? = null,
)
