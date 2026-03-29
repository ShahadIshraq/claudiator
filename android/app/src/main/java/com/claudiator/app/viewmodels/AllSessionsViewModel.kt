package com.claudiator.app.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.claudiator.app.models.Session
import com.claudiator.app.services.ApiClient
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class AllSessionsUiState(
    val sessions: List<Session> = emptyList(),
    val isLoading: Boolean = false,
    val isLoadingMore: Boolean = false,
    val hasMore: Boolean = false,
    val currentOffset: Int = 0,
    val error: String? = null,
    val filter: SessionFilter = SessionFilter.ACTIVE,
    val isGroupedByDevice: Boolean = false,
    val expandedDevices: Set<String> = emptySet(),
    val groupedSessions: Map<String, List<Session>> = emptyMap(),
)

class AllSessionsViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(AllSessionsUiState())
    val uiState: StateFlow<AllSessionsUiState> = _uiState.asStateFlow()

    fun setFilter(filter: SessionFilter) { _uiState.update { it.copy(filter = filter) } }

    fun refresh(apiClient: ApiClient) {
        if (_uiState.value.sessions.isEmpty()) _uiState.update { it.copy(isLoading = true) }
        viewModelScope.launch {
            try {
                val excludeEnded = _uiState.value.filter == SessionFilter.ACTIVE
                val result = apiClient.fetchAllSessionsPage(excludeEnded = excludeEnded, limit = 50, offset = 0)
                _uiState.update { state ->
                    val grouped = if (state.isGroupedByDevice) groupSessions(result.sessions) else emptyMap()
                    val expanded = if (state.isGroupedByDevice) autoExpand(grouped) else state.expandedDevices
                    state.copy(
                        sessions = result.sessions,
                        hasMore = result.hasMore,
                        currentOffset = result.nextOffset,
                        isLoading = false,
                        error = null,
                        groupedSessions = grouped,
                        expandedDevices = expanded,
                    )
                }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, error = e.message) }
            }
        }
    }

    fun loadMore(apiClient: ApiClient) {
        val state = _uiState.value
        if (!state.hasMore || state.isLoadingMore) return
        _uiState.update { it.copy(isLoadingMore = true) }
        viewModelScope.launch {
            try {
                val excludeEnded = state.filter == SessionFilter.ACTIVE
                val result = apiClient.fetchAllSessionsPage(
                    excludeEnded = excludeEnded, limit = 50, offset = state.currentOffset,
                )
                _uiState.update { current ->
                    val existingIds = current.sessions.map { it.sessionId }.toSet()
                    val newSessions = result.sessions.filter { it.sessionId !in existingIds }
                    val allSessions = current.sessions + newSessions
                    val grouped = if (current.isGroupedByDevice) groupSessions(allSessions) else emptyMap()
                    current.copy(
                        sessions = allSessions,
                        hasMore = result.hasMore,
                        currentOffset = result.nextOffset,
                        isLoadingMore = false,
                        error = null,
                        groupedSessions = grouped,
                    )
                }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoadingMore = false, error = e.message) }
            }
        }
    }

    fun toggleGrouping() {
        _uiState.update { state ->
            val newGrouped = !state.isGroupedByDevice
            val grouped = if (newGrouped) groupSessions(state.sessions) else emptyMap()
            val expanded = if (newGrouped) autoExpand(grouped) else emptySet()
            state.copy(isGroupedByDevice = newGrouped, groupedSessions = grouped, expandedDevices = expanded)
        }
    }

    fun toggleDevice(deviceId: String) {
        _uiState.update { state ->
            val expanded = if (deviceId in state.expandedDevices) {
                state.expandedDevices - deviceId
            } else {
                state.expandedDevices + deviceId
            }
            state.copy(expandedDevices = expanded)
        }
    }

    private fun groupSessions(sessions: List<Session>): Map<String, List<Session>> =
        sessions.groupBy { it.deviceId }

    private fun autoExpand(grouped: Map<String, List<Session>>): Set<String> =
        grouped.filter { (_, sessions) -> sessions.any { it.status != "ended" } }.keys
}
