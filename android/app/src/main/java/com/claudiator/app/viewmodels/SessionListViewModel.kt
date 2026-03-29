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

enum class SessionFilter(val label: String) {
    ACTIVE("Active"),
    ALL("All"),
}

data class SessionListUiState(
    val sessions: List<Session> = emptyList(),
    val isLoading: Boolean = false,
    val error: String? = null,
    val filter: SessionFilter = SessionFilter.ACTIVE,
)

class SessionListViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(SessionListUiState())
    val uiState: StateFlow<SessionListUiState> = _uiState.asStateFlow()

    fun setFilter(filter: SessionFilter) { _uiState.update { it.copy(filter = filter) } }

    fun refresh(apiClient: ApiClient, deviceId: String) {
        if (_uiState.value.sessions.isEmpty()) _uiState.update { it.copy(isLoading = true) }
        viewModelScope.launch {
            try {
                val all = apiClient.fetchSessions(deviceId)
                val filtered = if (_uiState.value.filter == SessionFilter.ACTIVE) {
                    all.filter { it.status != "ended" }
                } else all
                _uiState.update { it.copy(sessions = filtered, isLoading = false, error = null) }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, error = e.message) }
            }
        }
    }
}
