package com.claudiator.app.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.claudiator.app.services.ApiClient
import com.claudiator.app.services.ApiError
import com.claudiator.app.util.URLValidator
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class SetupUiState(
    val serverUrl: String = "",
    val apiKey: String = "",
    val isLoading: Boolean = false,
    val error: String? = null,
    val connectionSuccess: Boolean = false,
)

class SetupViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(SetupUiState())
    val uiState: StateFlow<SetupUiState> = _uiState.asStateFlow()

    fun updateServerUrl(url: String) { _uiState.update { it.copy(serverUrl = url) } }
    fun updateApiKey(key: String) { _uiState.update { it.copy(apiKey = key) } }

    fun connect(apiClient: ApiClient) {
        val cleanedUrl = URLValidator.cleanAndValidate(_uiState.value.serverUrl)
        if (cleanedUrl == null) {
            _uiState.update { it.copy(error = "Invalid URL format") }
            return
        }
        _uiState.update { it.copy(isLoading = true, error = null) }
        viewModelScope.launch {
            try {
                apiClient.configure(cleanedUrl, _uiState.value.apiKey)
                apiClient.ping()
                _uiState.update { it.copy(isLoading = false, connectionSuccess = true) }
            } catch (e: ApiError.Unauthorized) {
                apiClient.disconnect()
                _uiState.update { it.copy(isLoading = false, error = "Authentication Failed\n\nThe API key is incorrect. Please check your API key and try again.") }
            } catch (e: ApiError.ServerError) {
                apiClient.disconnect()
                _uiState.update { it.copy(isLoading = false, error = "Server Error (${e.code})\n\nThe server returned an error. Make sure the server is running correctly.") }
            } catch (e: Exception) {
                apiClient.disconnect()
                val msg = e.message ?: "Unknown error"
                val error = when {
                    msg.contains("timeout", ignoreCase = true) -> "Connection Timeout\n\nThe server didn't respond in time."
                    msg.contains("connect", ignoreCase = true) || msg.contains("refused", ignoreCase = true) ->
                        "Cannot Reach Server\n\nUnable to connect to: $cleanedUrl\n\nMake sure:\n- The server is running\n- The URL is correct\n- You're on the same network (for local servers)"
                    msg.contains("SSL", ignoreCase = true) || msg.contains("TLS", ignoreCase = true) ->
                        "HTTPS Connection Failed\n\nFor local development servers, use http:// instead of https://\n\nExample: http://192.168.1.5:3000"
                    else -> "Network Error\n\n$msg"
                }
                _uiState.update { it.copy(isLoading = false, error = error) }
            }
        }
    }
}
