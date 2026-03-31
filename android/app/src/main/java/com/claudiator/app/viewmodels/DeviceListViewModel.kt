package com.claudiator.app.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.claudiator.app.models.Device
import com.claudiator.app.services.ApiClient
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class SessionStatusCounts(
    val active: Int = 0,
    val waitingInput: Int = 0,
    val waitingPermission: Int = 0,
    val idle: Int = 0,
    val ended: Int = 0,
) {
    val totalActive: Int get() = active + waitingInput + waitingPermission + idle
}

data class DeviceListUiState(
    val devices: List<Device> = emptyList(),
    val statusCounts: Map<String, SessionStatusCounts> = emptyMap(),
    val isLoading: Boolean = false,
    val isRefreshing: Boolean = false,
    val error: String? = null,
)

class DeviceListViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(DeviceListUiState())
    val uiState: StateFlow<DeviceListUiState> = _uiState.asStateFlow()

    fun refresh(apiClient: ApiClient): Job {
        val isInitial = _uiState.value.devices.isEmpty()
        _uiState.update { it.copy(isLoading = isInitial, isRefreshing = !isInitial) }
        return viewModelScope.launch {
            try {
                val devicesDeferred = async { apiClient.fetchDevices() }
                val sessionsDeferred = async { apiClient.fetchAllSessions() }
                val devices = devicesDeferred.await()
                val sessions = sessionsDeferred.await()

                val counts = mutableMapOf<String, SessionStatusCounts>()
                for (s in sessions) {
                    val c = counts.getOrDefault(s.deviceId, SessionStatusCounts())
                    counts[s.deviceId] = when (s.status) {
                        "active" -> c.copy(active = c.active + 1)
                        "waiting_for_input" -> c.copy(waitingInput = c.waitingInput + 1)
                        "waiting_for_permission" -> c.copy(waitingPermission = c.waitingPermission + 1)
                        "idle" -> c.copy(idle = c.idle + 1)
                        "ended" -> c.copy(ended = c.ended + 1)
                        else -> c
                    }
                }
                _uiState.update { it.copy(devices = devices, statusCounts = counts, isLoading = false, isRefreshing = false, error = null) }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, isRefreshing = false, error = e.message) }
            }
        }
    }
}
