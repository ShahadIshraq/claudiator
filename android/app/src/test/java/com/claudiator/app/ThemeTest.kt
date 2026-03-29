package com.claudiator.app

import androidx.compose.ui.graphics.Color
import com.claudiator.app.ui.theme.*
import org.junit.Assert.*
import org.junit.Test

class ThemeTest {

    private val theme = StandardTheme

    @Test
    fun `statusColor returns correct color for known statuses`() {
        assertEquals(theme.statusActive, theme.statusColor("active"))
        assertEquals(theme.statusWaitingInput, theme.statusColor("waiting_for_input"))
        assertEquals(theme.statusWaitingPermission, theme.statusColor("waiting_for_permission"))
        assertEquals(theme.statusIdle, theme.statusColor("idle"))
        assertEquals(theme.statusEnded, theme.statusColor("ended"))
    }

    @Test
    fun `statusColor returns gray for unknown status`() {
        assertEquals(Color.Gray, theme.statusColor("unknown_status"))
        assertEquals(Color.Gray, theme.statusColor(""))
    }

    @Test
    fun `platformColor returns correct color for platforms`() {
        assertEquals(theme.platformMac, theme.platformColor("mac"))
        assertEquals(theme.platformMac, theme.platformColor("macos"))
        assertEquals(theme.platformMac, theme.platformColor("darwin"))
        assertEquals(theme.platformLinux, theme.platformColor("linux"))
        assertEquals(theme.platformWindows, theme.platformColor("windows"))
    }

    @Test
    fun `platformColor is case insensitive`() {
        assertEquals(theme.platformMac, theme.platformColor("MAC"))
        assertEquals(theme.platformLinux, theme.platformColor("Linux"))
        assertEquals(theme.platformWindows, theme.platformColor("WINDOWS"))
        assertEquals(theme.platformMac, theme.platformColor("DaRwIn"))
    }

    @Test
    fun `platformColor returns default for unknown platform`() {
        assertEquals(theme.platformDefault, theme.platformColor("unknown"))
        assertEquals(theme.platformDefault, theme.platformColor(""))
    }

    @Test
    fun `eventColor returns correct color for known events`() {
        assertEquals(theme.eventSessionStart, theme.eventColor("SessionStart"))
        assertEquals(theme.eventSessionEnd, theme.eventColor("SessionEnd"))
        assertEquals(theme.eventStop, theme.eventColor("Stop"))
        assertEquals(theme.eventNotification, theme.eventColor("Notification"))
        assertEquals(theme.eventUserPromptSubmit, theme.eventColor("UserPromptSubmit"))
    }

    @Test
    fun `eventColor returns default for unknown event`() {
        assertEquals(theme.eventDefault, theme.eventColor("UnknownEvent"))
        assertEquals(theme.eventDefault, theme.eventColor(""))
    }

    @Test
    fun `eventColor is case sensitive`() {
        assertEquals(theme.eventSessionStart, theme.eventColor("SessionStart"))
        assertEquals(theme.eventDefault, theme.eventColor("sessionstart"))
    }

    @Test
    fun `constants have expected values`() {
        assertEquals(12f, AppThemeDefaults.cardCornerRadius)
        assertEquals(0.3f, AppThemeDefaults.cardBorderOpacity)
        assertEquals(0.5f, AppThemeDefaults.cardBorderWidth)
    }

    @Test
    fun `all themes list contains 4 themes`() {
        assertEquals(4, allThemes.size)
        assertEquals("standard", allThemes[0].id)
        assertEquals("neon_ops", allThemes[1].id)
        assertEquals("solarized", allThemes[2].id)
        assertEquals("arctic", allThemes[3].id)
    }

    @Test
    fun `each theme has 4 preview colors`() {
        for (t in allThemes) {
            assertEquals("${t.name} should have 4 preview colors", 4, t.preview.size)
        }
    }
}
