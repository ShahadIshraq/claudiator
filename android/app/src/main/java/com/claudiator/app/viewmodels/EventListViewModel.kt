package com.claudiator.app.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.claudiator.app.models.Event
import com.claudiator.app.models.Session
import com.claudiator.app.services.ApiClient
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class EventListUiState(
    val events: List<Event> = emptyList(),
    val session: Session? = null,
    val isLoading: Boolean = false,
    val isRefreshing: Boolean = false,
    val error: String? = null,
)

class EventListViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(EventListUiState())
    val uiState: StateFlow<EventListUiState> = _uiState.asStateFlow()

    fun refresh(apiClient: ApiClient, sessionId: String): Job {
        val isInitial = _uiState.value.events.isEmpty()
        _uiState.update { it.copy(isLoading = isInitial, isRefreshing = !isInitial) }
        return viewModelScope.launch {
            try {
                val eventsDeferred = async { apiClient.fetchEvents(sessionId) }
                val sessionDeferred = async {
                    runCatching { apiClient.fetchAllSessions() }
                        .getOrDefault(emptyList())
                        .firstOrNull { it.sessionId == sessionId }
                }
                val events = eventsDeferred.await()
                val session = sessionDeferred.await()
                _uiState.update { it.copy(events = events, session = session, isLoading = false, isRefreshing = false, error = null) }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, isRefreshing = false, error = e.message) }
            }
        }
    }
}
