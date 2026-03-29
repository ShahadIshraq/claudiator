package com.claudiator.app.ui.sessions

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.layout.*
import androidx.compose.material3.HorizontalDivider
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.claudiator.app.models.Session
import com.claudiator.app.ui.components.ThemedCard

@Composable
fun DeviceGroupCard(
    deviceId: String,
    deviceName: String,
    platform: String,
    sessions: List<Session>,
    isExpanded: Boolean,
    onToggle: () -> Unit,
    onSessionClick: (String) -> Unit,
) {
    ThemedCard(modifier = Modifier.fillMaxWidth()) {
        DeviceGroupHeader(
            deviceId = deviceId,
            deviceName = deviceName,
            platform = platform,
            sessions = sessions,
            isExpanded = isExpanded,
            onToggle = onToggle,
        )

        AnimatedVisibility(
            visible = isExpanded,
            enter = expandVertically(),
            exit = shrinkVertically(),
        ) {
            Column {
                sessions.forEachIndexed { index, session ->
                    if (index > 0) {
                        HorizontalDivider(
                            modifier = Modifier.padding(start = 22.dp),
                            thickness = 0.5.dp,
                        )
                    }
                    SessionRow(
                        session = session,
                        onClick = { onSessionClick(session.sessionId) },
                    )
                }
            }
        }
    }
}
