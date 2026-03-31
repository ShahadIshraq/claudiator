package com.claudiator.app

import com.claudiator.app.models.Session
import com.claudiator.app.viewmodels.*
import org.junit.Assert.*
import org.junit.Test

class ViewModelTest {

    private fun session(id: String, deviceId: String, status: String) = Session(
        sessionId = id, deviceId = deviceId,
        startedAt = "2024-01-15T10:00:00Z", lastEvent = "2024-01-15T11:00:00Z",
        status = status,
    )

    // -- SessionStatusCounts --

    @Test
    fun `SessionStatusCounts initializes with zeros`() {
        val c = SessionStatusCounts()
        assertEquals(0, c.active); assertEquals(0, c.waitingInput)
        assertEquals(0, c.waitingPermission); assertEquals(0, c.idle)
        assertEquals(0, c.ended); assertEquals(0, c.totalActive)
    }

    @Test
    fun `SessionStatusCounts totalActive excludes ended`() {
        val c = SessionStatusCounts(active = 2, waitingInput = 1, waitingPermission = 1, idle = 3, ended = 5)
        assertEquals(7, c.totalActive)
    }

    @Test
    fun `SessionStatusCounts totalActive with only ended`() {
        val c = SessionStatusCounts(ended = 10)
        assertEquals(0, c.totalActive)
    }

    // -- AllSessionsViewModel --

    @Test
    fun `AllSessionsViewModel initializes with defaults`() {
        val vm = AllSessionsViewModel()
        val state = vm.uiState.value
        assertTrue(state.sessions.isEmpty())
        assertFalse(state.isLoading)
        assertNull(state.error)
        assertEquals(SessionFilter.ACTIVE, state.filter)
        assertFalse(state.isGroupedByDevice)
        assertTrue(state.expandedDevices.isEmpty())
        assertTrue(state.groupedSessions.isEmpty())
    }

    @Test
    fun `AllSessionsViewModel toggleGrouping changes state`() {
        val vm = AllSessionsViewModel()
        assertFalse(vm.uiState.value.isGroupedByDevice)
        vm.toggleGrouping()
        assertTrue(vm.uiState.value.isGroupedByDevice)
        vm.toggleGrouping()
        assertFalse(vm.uiState.value.isGroupedByDevice)
    }

    @Test
    fun `AllSessionsViewModel toggleDevice expands and collapses`() {
        val vm = AllSessionsViewModel()
        assertFalse(vm.uiState.value.expandedDevices.contains("dev1"))
        vm.toggleDevice("dev1")
        assertTrue(vm.uiState.value.expandedDevices.contains("dev1"))
        vm.toggleDevice("dev1")
        assertFalse(vm.uiState.value.expandedDevices.contains("dev1"))
    }

    // -- SessionFilter --

    @Test
    fun `SessionFilter has correct labels`() {
        assertEquals("Active", SessionFilter.ACTIVE.label)
        assertEquals("All", SessionFilter.ALL.label)
        assertEquals(2, SessionFilter.entries.size)
    }

    // -- Session grouping logic --

    @Test
    fun `sessions can be grouped by device ID`() {
        val sessions = listOf(
            session("s1", "dev1", "active"),
            session("s2", "dev1", "idle"),
            session("s3", "dev2", "active"),
        )
        val grouped = sessions.groupBy { it.deviceId }
        assertEquals(2, grouped.size)
        assertEquals(2, grouped["dev1"]?.size)
        assertEquals(1, grouped["dev2"]?.size)
    }

    @Test
    fun `active session detection logic`() {
        val sessions = listOf(session("s1", "d1", "active"), session("s2", "d1", "ended"))
        assertTrue(sessions.any { it.status != "ended" })

        val ended = listOf(session("s1", "d1", "ended"), session("s2", "d1", "ended"))
        assertFalse(ended.any { it.status != "ended" })
    }

    // -- Status count aggregation --

    @Test
    fun `status count aggregation`() {
        val sessions = listOf(
            session("s1", "d1", "active"),
            session("s2", "d1", "active"),
            session("s3", "d1", "waiting_for_input"),
            session("s4", "d1", "waiting_for_permission"),
            session("s5", "d1", "idle"),
            session("s6", "d1", "ended"),
        )
        var c = SessionStatusCounts()
        for (s in sessions) {
            c = when (s.status) {
                "active" -> c.copy(active = c.active + 1)
                "waiting_for_input" -> c.copy(waitingInput = c.waitingInput + 1)
                "waiting_for_permission" -> c.copy(waitingPermission = c.waitingPermission + 1)
                "idle" -> c.copy(idle = c.idle + 1)
                "ended" -> c.copy(ended = c.ended + 1)
                else -> c
            }
        }
        assertEquals(2, c.active); assertEquals(1, c.waitingInput)
        assertEquals(1, c.waitingPermission); assertEquals(1, c.idle)
        assertEquals(1, c.ended); assertEquals(5, c.totalActive)
    }
}
