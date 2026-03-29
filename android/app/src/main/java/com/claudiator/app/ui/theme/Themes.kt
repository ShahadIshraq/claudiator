package com.claudiator.app.ui.theme

import androidx.compose.ui.graphics.Color

val allThemes: List<AppTheme> = listOf(StandardTheme, NeonOpsTheme, SolarizedTheme, ArcticTheme)

object StandardTheme : AppTheme {
    override val id = "standard"
    override val name = "Standard"
    override val preview = listOf(Color(0xFF4CAF50), Color(0xFF2196F3), Color(0xFFFF9800), Color(0xFFF44336))
    override val statusActive = Color(0xFF4CAF50)
    override val statusWaitingInput = Color(0xFFFF9800)
    override val statusWaitingPermission = Color(0xFFF44336)
    override val statusIdle = Color.Gray
    override val statusEnded = Color.Gray
    override val platformMac = Color(0xFF2196F3)
    override val platformLinux = Color(0xFFFF9800)
    override val platformWindows = Color(0xFF00BCD4)
    override val platformDefault = Color.Gray
    override val eventSessionStart = Color(0xFF4CAF50)
    override val eventSessionEnd = Color.Gray
    override val eventStop = Color(0xFFFF9800)
    override val eventNotification = Color(0xFFF44336)
    override val eventUserPromptSubmit = Color(0xFF2196F3)
    override val eventDefault = Color.Gray
    override val serverConnected = Color(0xFF4CAF50)
    override val serverDisconnected = Color(0xFFF44336)
    override val uiError = Color(0xFFF44336)
    override val uiAccent = Color(0xFF2196F3)
    override val uiTint = Color(0xFF2196F3)
    override fun pageBackground(isDark: Boolean) = if (isDark) Color(0xFF121212) else Color(0xFFF2F2F7)
    override fun cardBackground(isDark: Boolean) = if (isDark) Color(0xFF1C1C1E) else Color(0xFFFFFFFF)
    override fun cardBorder(isDark: Boolean) = if (isDark) Color(0xFF333333) else Color(0xFFCCCCCC)
}

object NeonOpsTheme : AppTheme {
    override val id = "neon_ops"
    override val name = "Neon Ops"
    override val preview = listOf(Color(0xFF00FF99), Color(0xFFFF4080), Color(0xFFFFC200), Color(0xFF00BFFF))
    override val statusActive = Color(0xFF00FF99)
    override val statusWaitingInput = Color(0xFFFFC200)
    override val statusWaitingPermission = Color(0xFFFF4080)
    override val statusIdle = Color(0xFF888888)
    override val statusEnded = Color(0xFF666666)
    override val platformMac = Color(0xFF00BFFF)
    override val platformLinux = Color(0xFFFFC200)
    override val platformWindows = Color(0xFF00FF99)
    override val platformDefault = Color(0xFF888888)
    override val eventSessionStart = Color(0xFF00FF99)
    override val eventSessionEnd = Color(0xFF666666)
    override val eventStop = Color(0xFFFFC200)
    override val eventNotification = Color(0xFFFF4080)
    override val eventUserPromptSubmit = Color(0xFF00BFFF)
    override val eventDefault = Color(0xFF888888)
    override val serverConnected = Color(0xFF00FF99)
    override val serverDisconnected = Color(0xFFFF4080)
    override val uiError = Color(0xFFFF4080)
    override val uiAccent = Color(0xFF00BFFF)
    override val uiTint = Color(0xFF00FF99)
    override fun pageBackground(isDark: Boolean) = if (isDark) Color(0xFF050D08) else Color(0xFFE0F2E9)
    override fun cardBackground(isDark: Boolean) = if (isDark) Color(0xFF0F1F14) else Color(0xFFF8FFF9)
    override fun cardBorder(isDark: Boolean) = if (isDark) Color(0xFF009959) else Color(0xFF00CC80)
}

object SolarizedTheme : AppTheme {
    override val id = "solarized"
    override val name = "Solarized"
    override val preview = listOf(Color(0xFF859900), Color(0xFF268BD2), Color(0xFFB58900), Color(0xFFDC322F))
    override val statusActive = Color(0xFF859900)
    override val statusWaitingInput = Color(0xFFB58900)
    override val statusWaitingPermission = Color(0xFFDC322F)
    override val statusIdle = Color(0xFF93A1A1)
    override val statusEnded = Color(0xFF839496)
    override val platformMac = Color(0xFF268BD2)
    override val platformLinux = Color(0xFFCB4B16)
    override val platformWindows = Color(0xFF2AA198)
    override val platformDefault = Color(0xFF93A1A1)
    override val eventSessionStart = Color(0xFF859900)
    override val eventSessionEnd = Color(0xFF839496)
    override val eventStop = Color(0xFFCB4B16)
    override val eventNotification = Color(0xFFDC322F)
    override val eventUserPromptSubmit = Color(0xFF268BD2)
    override val eventDefault = Color(0xFF93A1A1)
    override val serverConnected = Color(0xFF859900)
    override val serverDisconnected = Color(0xFFDC322F)
    override val uiError = Color(0xFFDC322F)
    override val uiAccent = Color(0xFF268BD2)
    override val uiTint = Color(0xFF268BD2)
    override fun pageBackground(isDark: Boolean) = if (isDark) Color(0xFF002129) else Color(0xFFE3DEC9)
    override fun cardBackground(isDark: Boolean) = if (isDark) Color(0xFF083642) else Color(0xFFFAF5E3)
    override fun cardBorder(isDark: Boolean) = if (isDark) Color(0xFF124A57) else Color(0xFFD1C7B0)
}

object ArcticTheme : AppTheme {
    override val id = "arctic"
    override val name = "Arctic"
    override val preview = listOf(Color(0xFF2EB887), Color(0xFF3B82F6), Color(0xFF8B95A5), Color(0xFFEF4444))
    override val statusActive = Color(0xFF2EB887)
    override val statusWaitingInput = Color(0xFFF59E0B)
    override val statusWaitingPermission = Color(0xFFEF4444)
    override val statusIdle = Color(0xFF8B95A5)
    override val statusEnded = Color(0xFF6B7280)
    override val platformMac = Color(0xFF3B82F6)
    override val platformLinux = Color(0xFFF59E0B)
    override val platformWindows = Color(0xFF06B6D4)
    override val platformDefault = Color(0xFF8B95A5)
    override val eventSessionStart = Color(0xFF2EB887)
    override val eventSessionEnd = Color(0xFF6B7280)
    override val eventStop = Color(0xFFF59E0B)
    override val eventNotification = Color(0xFFEF4444)
    override val eventUserPromptSubmit = Color(0xFF3B82F6)
    override val eventDefault = Color(0xFF8B95A5)
    override val serverConnected = Color(0xFF2EB887)
    override val serverDisconnected = Color(0xFFEF4444)
    override val uiError = Color(0xFFEF4444)
    override val uiAccent = Color(0xFF3B82F6)
    override val uiTint = Color(0xFF2EB887)
    override fun pageBackground(isDark: Boolean) = if (isDark) Color(0xFF080F1A) else Color(0xFFDEEBFA)
    override fun cardBackground(isDark: Boolean) = if (isDark) Color(0xFF142133) else Color(0xFFF5FAFF)
    override fun cardBorder(isDark: Boolean) = if (isDark) Color(0xFF213045) else Color(0xFFC7DBEE)
}
