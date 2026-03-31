package com.claudiator.app.services

import com.claudiator.app.models.*
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.engine.android.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.json.Json

sealed class ApiError(message: String) : Exception(message) {
    object NotConfigured : ApiError("Server not configured")
    object InvalidURL : ApiError("Invalid server URL")
    object Unauthorized : ApiError("Invalid API key")
    data class ServerError(val code: Int) : ApiError("Server error ($code)")
    data class NetworkError(override val cause: Throwable) : ApiError(cause.message ?: "Network error")
    data class DecodingError(override val cause: Throwable) : ApiError("Data error: ${cause.message}")
}

class ApiClient(private val secureStorage: SecureStorage) {

    private val json = Json { ignoreUnknownKeys = true }

    private val client = HttpClient(Android) {
        install(ContentNegotiation) {
            json(json)
        }
        install(HttpTimeout) {
            requestTimeoutMillis = 15_000
            connectTimeoutMillis = 15_000
            socketTimeoutMillis = 30_000
        }
    }

    private val _isConfigured = MutableStateFlow(secureStorage.isConfigured)
    val isConfigured: StateFlow<Boolean> = _isConfigured.asStateFlow()

    fun refreshConfigState() {
        _isConfigured.value = secureStorage.isConfigured
    }

    private val retryableExceptions = setOf(
        "connect", "timeout", "timed out", "network", "refused",
    )

    private suspend inline fun <reified T> request(
        path: String,
        method: HttpMethod = HttpMethod.Get,
        body: Any? = null,
    ): T {
        val baseUrl = secureStorage.serverUrl
        val apiKey = secureStorage.apiKey
        if (baseUrl.isNullOrEmpty() || apiKey.isNullOrEmpty()) throw ApiError.NotConfigured

        val urlString = if (baseUrl.endsWith("/")) baseUrl + path.removePrefix("/") else baseUrl + path

        var lastError: Exception? = null
        for (attempt in 0 until 3) {
            if (attempt > 0) delay(500L * attempt)
            try {
                val response = client.request(urlString) {
                    this.method = method
                    header("Authorization", "Bearer $apiKey")
                    contentType(ContentType.Application.Json)
                    if (body != null) setBody(body)
                }
                when (response.status.value) {
                    in 200..299 -> return response.body<T>()
                    401 -> throw ApiError.Unauthorized
                    else -> throw ApiError.ServerError(response.status.value)
                }
            } catch (e: ApiError) {
                throw e
            } catch (e: Exception) {
                val msg = (e.message ?: "").lowercase()
                if (retryableExceptions.any { msg.contains(it) }) {
                    lastError = e
                } else {
                    throw ApiError.NetworkError(e)
                }
            }
        }
        throw ApiError.NetworkError(lastError ?: Exception("Unknown error"))
    }

    suspend fun ping(): PingResponse = request("/api/v1/ping")

    suspend fun fetchDevices(): List<Device> =
        request<DevicesResponse>("/api/v1/devices").devices

    suspend fun fetchSessions(deviceId: String, status: String? = null, limit: Int? = null): List<Session> {
        val params = buildList {
            if (status != null) add("status=$status")
            if (limit != null) add("limit=$limit")
        }
        val query = if (params.isNotEmpty()) "?" + params.joinToString("&") else ""
        return request<SessionsResponse>("/api/v1/devices/$deviceId/sessions$query").sessions
    }

    suspend fun fetchAllSessions(status: String? = null, limit: Int? = null): List<Session> {
        val params = buildList {
            if (status != null) add("status=$status")
            if (limit != null) add("limit=$limit")
        }
        val query = if (params.isNotEmpty()) "?" + params.joinToString("&") else ""
        return request<SessionsResponse>("/api/v1/sessions$query").sessions
    }

    suspend fun fetchAllSessionsPage(
        excludeEnded: Boolean = false,
        limit: Int = 50,
        offset: Int = 0,
    ): SessionListPage {
        val params = buildList {
            if (excludeEnded) add("exclude_ended=true")
            add("limit=$limit")
            add("offset=$offset")
        }
        return request("/api/v1/sessions?" + params.joinToString("&"))
    }

    suspend fun fetchEvents(sessionId: String, limit: Int? = null): List<Event> {
        val query = if (limit != null) "?limit=$limit" else ""
        return request<EventsResponse>("/api/v1/sessions/$sessionId/events$query").events
    }

    suspend fun registerPushToken(token: String) {
        @kotlinx.serialization.Serializable
        data class Body(
            val platform: String,
            @kotlinx.serialization.SerialName("push_token") val pushToken: String,
            val sandbox: Boolean,
        )
        request<Map<String, String>>(
            "/api/v1/push/register",
            method = HttpMethod.Post,
            body = Body(platform = "android", pushToken = token, sandbox = false),
        )
    }

    suspend fun fetchNotifications(after: String? = null, limit: Int? = null): List<AppNotification> {
        val params = buildList {
            if (after != null) add("after=$after")
            if (limit != null) add("limit=$limit")
        }
        val query = if (params.isNotEmpty()) "?" + params.joinToString("&") else ""
        return request<NotificationsResponse>("/api/v1/notifications$query").notifications
    }

    suspend fun acknowledgeNotifications(ids: List<String>) {
        @kotlinx.serialization.Serializable
        data class Body(@kotlinx.serialization.SerialName("notification_ids") val notificationIds: List<String>)
        request<Map<String, String>>(
            "/api/v1/notifications/ack",
            method = HttpMethod.Post,
            body = Body(notificationIds = ids),
        )
    }

    fun configure(url: String, apiKey: String) {
        secureStorage.serverUrl = url
        secureStorage.apiKey = apiKey
        refreshConfigState()
    }

    fun disconnect() {
        secureStorage.clear()
        refreshConfigState()
    }
}
