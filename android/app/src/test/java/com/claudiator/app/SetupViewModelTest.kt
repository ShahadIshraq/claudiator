package com.claudiator.app

import com.claudiator.app.viewmodels.SetupViewModel
import org.junit.Assert.*
import org.junit.Test

class SetupViewModelTest {

    @Test
    fun `initial state is empty`() {
        val vm = SetupViewModel()
        val state = vm.uiState.value
        assertEquals("", state.serverUrl)
        assertEquals("", state.apiKey)
        assertFalse(state.isLoading)
        assertNull(state.error)
        assertFalse(state.connectionSuccess)
    }

    @Test
    fun `updateServerUrl updates state`() {
        val vm = SetupViewModel()
        vm.updateServerUrl("https://example.com")
        assertEquals("https://example.com", vm.uiState.value.serverUrl)
    }

    @Test
    fun `updateApiKey updates state`() {
        val vm = SetupViewModel()
        vm.updateApiKey("test-key")
        assertEquals("test-key", vm.uiState.value.apiKey)
    }
}
