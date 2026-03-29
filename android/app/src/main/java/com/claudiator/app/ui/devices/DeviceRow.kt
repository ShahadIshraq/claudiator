package com.claudiator.app.ui.devices

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.claudiator.app.models.Device
import com.claudiator.app.ui.components.PlatformIcon
import com.claudiator.app.ui.components.StatusBadge
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.util.relativeTime
import com.claudiator.app.viewmodels.SessionStatusCounts

@Composable
fun DeviceRow(
    device: Device,
    statusCounts: SessionStatusCounts?,
    onClick: () -> Unit,
) {
    val theme = LocalAppTheme.current

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        PlatformIcon(
            platform = device.platform,
            modifier = Modifier.size(28.dp),
        )

        Spacer(Modifier.width(12.dp))

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = device.deviceName,
                style = MaterialTheme.typography.bodyLarge,
            )
            Spacer(Modifier.height(2.dp))
            Text(
                text = "last active ${relativeTime(device.lastSeen)}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        if (statusCounts != null) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                StatusBadge(
                    count = statusCounts.active,
                    label = "active",
                    color = theme.statusActive,
                )
                StatusBadge(
                    count = statusCounts.waitingInput + statusCounts.waitingPermission,
                    label = "waiting",
                    color = theme.statusWaitingInput,
                )
                StatusBadge(
                    count = statusCounts.idle,
                    label = "idle",
                    color = theme.statusIdle,
                )
            }
        }
    }
}
