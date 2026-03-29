package com.claudiator.app.ui.sessions

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.claudiator.app.models.Session
import com.claudiator.app.ui.components.PlatformIcon
import com.claudiator.app.ui.components.StatusBadge
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.statusColor
import com.claudiator.app.util.priorityStatus

@Composable
fun DeviceGroupHeader(
    deviceId: String,
    deviceName: String,
    platform: String,
    sessions: List<Session>,
    isExpanded: Boolean,
    onToggle: () -> Unit,
) {
    val theme = LocalAppTheme.current
    val priority = priorityStatus(sessions)
    val priorityColor = theme.statusColor(priority)

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onToggle)
            .padding(horizontal = 16.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        PlatformIcon(
            platform = platform,
            modifier = Modifier.size(22.dp),
        )

        Spacer(Modifier.width(10.dp))

        Text(
            text = deviceName,
            style = MaterialTheme.typography.titleSmall,
            modifier = Modifier.weight(1f),
        )

        Spacer(Modifier.width(8.dp))

        Text(
            text = "${sessions.size}",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        Spacer(Modifier.width(6.dp))

        StatusBadge(
            count = 1,
            label = priority.replace("_", " "),
            color = priorityColor,
        )

        Spacer(Modifier.width(4.dp))

        Icon(
            imageVector = if (isExpanded) Icons.Default.KeyboardArrowUp else Icons.Default.KeyboardArrowDown,
            contentDescription = if (isExpanded) "Collapse" else "Expand",
            modifier = Modifier.size(20.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
