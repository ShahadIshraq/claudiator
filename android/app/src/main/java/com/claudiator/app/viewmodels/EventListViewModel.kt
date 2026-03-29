package com.claudiator.app.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.claudiator.app.models.Event
import com.claudiator.app.services.ApiClient
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class EventListUiState(
    val events: List<Event> = emptyList(),
    val isLoading: Boolean = false,
    val error: String? = null,
)

class EventListViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(EventListUiState())
    val uiState: StateFlow<EventListUiState> = _uiState.asStateFlow()

    fun refresh(apiClient: ApiClient, sessionId: String) {
        if (_uiState.value.events.isEmpty()) _uiState.update { it.copy(isLoading = true) }
        viewModelScope.launch {
            try {
                val events = apiClient.fetchEvents(sessionId)
                _uiState.update { it.copy(events = events, isLoading = false, error = null) }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, error = e.message) }
            }
        }
    }
}
