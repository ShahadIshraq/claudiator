package com.claudiator.app.ui.sessions

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.claudiator.app.models.Event
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.eventColor
import com.claudiator.app.util.relativeTime

@Composable
fun EventRow(event: Event) {
    val theme = LocalAppTheme.current
    val color = theme.eventColor(event.hookEventName)

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp),
        verticalAlignment = Alignment.Top,
    ) {
        Box(
            modifier = Modifier
                .size(10.dp)
                .clip(CircleShape)
                .background(color)
                .align(Alignment.Top)
                .offset(y = 4.dp),
        )

        Spacer(Modifier.width(10.dp))

        Column(modifier = Modifier.weight(1f)) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                Text(
                    text = event.hookEventName,
                    style = MaterialTheme.typography.bodyMedium,
                    fontWeight = FontWeight.Medium,
                )

                if (event.toolName != null) {
                    Text(
                        text = event.toolName,
                        fontSize = 11.sp,
                        color = theme.uiTint,
                        modifier = Modifier
                            .background(
                                theme.uiTint.copy(alpha = 0.12f),
                                RoundedCornerShape(4.dp),
                            )
                            .padding(horizontal = 5.dp, vertical = 2.dp),
                    )
                }

                if (event.notificationType != null) {
                    Text(
                        text = event.notificationType,
                        fontSize = 11.sp,
                        color = theme.eventNotification,
                        modifier = Modifier
                            .background(
                                theme.eventNotification.copy(alpha = 0.12f),
                                RoundedCornerShape(4.dp),
                            )
                            .padding(horizontal = 5.dp, vertical = 2.dp),
                    )
                }
            }

            if (!event.message.isNullOrBlank()) {
                Spacer(Modifier.height(2.dp))
                Text(
                    text = event.message,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 3,
                )
            }
        }

        Spacer(Modifier.width(8.dp))

        Text(
            text = relativeTime(event.timestamp),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
