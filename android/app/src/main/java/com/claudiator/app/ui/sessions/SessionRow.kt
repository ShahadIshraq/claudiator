package com.claudiator.app.ui.sessions

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import com.claudiator.app.models.Session
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.statusColor
import com.claudiator.app.util.cwdShortDisplay
import com.claudiator.app.util.relativeTime
import com.claudiator.app.util.statusDisplayLabel

@Composable
fun SessionRow(
    session: Session,
    onClick: () -> Unit,
) {
    val theme = LocalAppTheme.current
    val color = theme.statusColor(session.status)

    val title = when {
        !session.title.isNullOrBlank() -> session.title
        !session.cwd.isNullOrBlank() -> cwdShortDisplay(session.cwd)
        else -> session.sessionId.take(8)
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 0.dp, vertical = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .size(12.dp)
                .clip(CircleShape)
                .background(color),
        )

        Spacer(Modifier.width(10.dp))

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
            )
            Spacer(Modifier.height(2.dp))
            Text(
                text = statusDisplayLabel(session.status),
                style = MaterialTheme.typography.bodySmall,
                color = color,
            )
        }

        Text(
            text = relativeTime(session.lastEvent),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
