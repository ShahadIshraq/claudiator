package com.claudiator.app.ui.theme

import androidx.compose.ui.graphics.Color

interface AppTheme {
    val id: String
    val name: String
    val preview: List<Color>

    val statusActive: Color
    val statusWaitingInput: Color
    val statusWaitingPermission: Color
    val statusIdle: Color
    val statusEnded: Color

    val platformMac: Color
    val platformLinux: Color
    val platformWindows: Color
    val platformDefault: Color

    val eventSessionStart: Color
    val eventSessionEnd: Color
    val eventStop: Color
    val eventNotification: Color
    val eventUserPromptSubmit: Color
    val eventDefault: Color

    val serverConnected: Color
    val serverDisconnected: Color

    val uiError: Color
    val uiAccent: Color
    val uiTint: Color

    fun pageBackground(isDark: Boolean): Color
    fun cardBackground(isDark: Boolean): Color
    fun cardBorder(isDark: Boolean): Color
}

fun AppTheme.statusColor(status: String): Color = when (status) {
    "active" -> statusActive
    "waiting_for_input" -> statusWaitingInput
    "waiting_for_permission" -> statusWaitingPermission
    "idle" -> statusIdle
    "ended" -> statusEnded
    else -> Color.Gray
}

fun AppTheme.platformColor(platform: String): Color = when (platform.lowercase()) {
    "mac", "macos", "darwin" -> platformMac
    "linux" -> platformLinux
    "windows" -> platformWindows
    else -> platformDefault
}

fun AppTheme.eventColor(hookEventName: String): Color = when (hookEventName) {
    "SessionStart" -> eventSessionStart
    "SessionEnd" -> eventSessionEnd
    "Stop" -> eventStop
    "Notification" -> eventNotification
    "UserPromptSubmit" -> eventUserPromptSubmit
    else -> eventDefault
}

object AppThemeDefaults {
    const val cardCornerRadius = 12f
    const val cardBorderOpacity = 0.3f
    const val cardBorderWidth = 0.5f
}
