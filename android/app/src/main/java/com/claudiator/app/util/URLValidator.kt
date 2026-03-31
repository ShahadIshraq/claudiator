package com.claudiator.app.util

import java.net.URI

object URLValidator {
    fun cleanAndValidate(input: String): String? {
        var url = input.trim()
        if (url.isEmpty()) return null
        if (url.endsWith("/")) url = url.dropLast(1)
        if (!url.startsWith("http")) {
            val isLocal = url.contains(".local") ||
                url.startsWith("localhost") ||
                url.startsWith("127.0.0.1")
            url = (if (isLocal) "http://" else "https://") + url
        }
        return try {
            URI(url).toURL()
            url
        } catch (_: Exception) {
            null
        }
    }
}
