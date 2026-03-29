package com.claudiator.app.util

import com.claudiator.app.models.Session
import java.time.Duration
import java.time.Instant
import java.time.format.DateTimeParseException

fun relativeTime(isoString: String): String {
    val instant = try {
        Instant.parse(isoString)
    } catch (_: DateTimeParseException) {
        return isoString
    }
    val now = Instant.now()
    val duration = Duration.between(instant, now)
    val seconds = duration.seconds
    return when {
        seconds < 0 -> "just now"
        seconds < 60 -> "${seconds}s ago"
        seconds < 3600 -> "${seconds / 60}m ago"
        seconds < 86400 -> "${seconds / 3600}h ago"
        seconds < 604800 -> "${seconds / 86400}d ago"
        else -> {
            val days = seconds / 86400
            "${days / 7}w ago"
        }
    }
}

fun cwdShortDisplay(cwd: String): String {
    val components = cwd.split("/")
    if (components.size >= 2) {
        val nonEmpty = components.filter { it.isNotEmpty() }
        if (nonEmpty.size >= 2) {
            return nonEmpty.takeLast(2).joinToString("/")
        }
    }
    val winComponents = cwd.split("\\")
    if (winComponents.size >= 2) {
        val nonEmpty = winComponents.filter { it.isNotEmpty() }
        if (nonEmpty.size >= 2) {
            return nonEmpty.takeLast(2).joinToString("/")
        }
    }
    return cwd
}

fun statusDisplayLabel(status: String): String = when (status) {
    "active" -> "Active"
    "waiting_for_input" -> "Waiting for Input"
    "waiting_for_permission" -> "Waiting for Permission"
    "idle" -> "Idle"
    "ended" -> "Ended"
    else -> status.replace("_", " ").split(" ").joinToString(" ") { word ->
        word.replaceFirstChar { it.uppercaseChar() }
    }
}

fun priorityStatus(sessions: List<Session>): String {
    if (sessions.any { it.status == "waiting_for_permission" }) return "waiting_for_permission"
    if (sessions.any { it.status == "waiting_for_input" }) return "waiting_for_input"
    if (sessions.any { it.status == "active" }) return "active"
    if (sessions.any { it.status == "idle" }) return "idle"
    return "ended"
}

fun platformIconName(platform: String): String = when (platform.lowercase()) {
    "mac", "macos", "darwin" -> "apple"
    "linux" -> "linux"
    "windows" -> "windows"
    else -> "monitor"
}
