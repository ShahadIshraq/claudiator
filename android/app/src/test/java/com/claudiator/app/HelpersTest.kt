package com.claudiator.app

import com.claudiator.app.models.Session
import com.claudiator.app.util.*
import org.junit.Assert.*
import org.junit.Test

class HelpersTest {

    @Test
    fun `cwdShortDisplay with Unix path`() {
        assertEquals("workspace/project", cwdShortDisplay("/Users/test/workspace/project"))
    }

    @Test
    fun `cwdShortDisplay with single component`() {
        assertEquals("/project", cwdShortDisplay("/project"))
    }

    @Test
    fun `cwdShortDisplay with Windows path`() {
        assertEquals("Documents/project", cwdShortDisplay("C:\\Users\\test\\Documents\\project"))
    }

    @Test
    fun `cwdShortDisplay with exactly two components`() {
        assertEquals("workspace/project", cwdShortDisplay("/workspace/project"))
    }

    @Test
    fun `statusDisplayLabel for known statuses`() {
        assertEquals("Active", statusDisplayLabel("active"))
        assertEquals("Waiting for Input", statusDisplayLabel("waiting_for_input"))
        assertEquals("Waiting for Permission", statusDisplayLabel("waiting_for_permission"))
        assertEquals("Idle", statusDisplayLabel("idle"))
        assertEquals("Ended", statusDisplayLabel("ended"))
    }

    @Test
    fun `statusDisplayLabel for unknown status`() {
        assertEquals("Custom Status", statusDisplayLabel("custom_status"))
    }

    private fun session(id: String, status: String) = Session(
        sessionId = id,
        deviceId = "dev1",
        startedAt = "2024-01-15T10:00:00Z",
        lastEvent = "2024-01-15T11:00:00Z",
        status = status,
    )

    @Test
    fun `priorityStatus returns waiting_for_permission when present`() {
        val sessions = listOf(
            session("1", "idle"),
            session("2", "waiting_for_permission"),
            session("3", "active"),
        )
        assertEquals("waiting_for_permission", priorityStatus(sessions))
    }

    @Test
    fun `priorityStatus returns waiting_for_input when no permission wait`() {
        val sessions = listOf(session("1", "idle"), session("2", "waiting_for_input"))
        assertEquals("waiting_for_input", priorityStatus(sessions))
    }

    @Test
    fun `priorityStatus returns active when no waiting states`() {
        val sessions = listOf(session("1", "active"), session("2", "idle"))
        assertEquals("active", priorityStatus(sessions))
    }

    @Test
    fun `priorityStatus returns idle when only idle and ended`() {
        val sessions = listOf(session("1", "idle"), session("2", "ended"))
        assertEquals("idle", priorityStatus(sessions))
    }

    @Test
    fun `priorityStatus returns ended for all ended sessions`() {
        val sessions = listOf(session("1", "ended"))
        assertEquals("ended", priorityStatus(sessions))
    }

    @Test
    fun `priorityStatus returns ended for empty list`() {
        assertEquals("ended", priorityStatus(emptyList()))
    }

    @Test
    fun `platformIconName returns correct name`() {
        assertEquals("apple", platformIconName("mac"))
        assertEquals("apple", platformIconName("macos"))
        assertEquals("apple", platformIconName("darwin"))
        assertEquals("linux", platformIconName("linux"))
        assertEquals("windows", platformIconName("windows"))
        assertEquals("monitor", platformIconName("unknown"))
    }

    @Test
    fun `platformIconName is case insensitive`() {
        assertEquals("apple", platformIconName("MAC"))
        assertEquals("linux", platformIconName("Linux"))
        assertEquals("windows", platformIconName("WINDOWS"))
    }
}
