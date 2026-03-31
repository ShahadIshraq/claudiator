package com.claudiator.app.ui.notifications

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.claudiator.app.services.ApiClient
import com.claudiator.app.services.AppNotificationManager
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NotificationListSheet(
    notificationManager: AppNotificationManager,
    apiClient: ApiClient,
    onDismiss: () -> Unit,
) {
    val state by notificationManager.state.collectAsState()
    val scope = rememberCoroutineScope()

    ModalBottomSheet(onDismissRequest = onDismiss) {
        // Header row
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = "Notifications",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.weight(1f),
            )
            TextButton(
                onClick = {
                    scope.launch {
                        val markedIds = notificationManager.markAllRead()
                        if (markedIds.isNotEmpty()) {
                            runCatching { apiClient.acknowledgeNotifications(markedIds) }
                        }
                    }
                },
            ) {
                Text("Mark All Read")
            }
        }

        HorizontalDivider(modifier = Modifier.padding(top = 4.dp))

        if (state.notifications.isEmpty()) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(48.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "No Notifications",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxWidth(),
                contentPadding = PaddingValues(bottom = 16.dp),
            ) {
                items(
                    items = state.notifications,
                    key = { it.notificationId },
                ) { notification ->
                    val isUnread = notification.notificationId !in state.readIds

                    val dismissState = rememberSwipeToDismissBoxState(
                        confirmValueChange = { value ->
                            if (value != SwipeToDismissBoxValue.Settled) {
                                scope.launch {
                                    val markedId = notificationManager.markNotificationRead(notification.notificationId)
                                    if (markedId != null) {
                                        runCatching { apiClient.acknowledgeNotifications(listOf(markedId)) }
                                    }
                                }
                                true
                            } else {
                                false
                            }
                        },
                    )

                    SwipeToDismissBox(
                        state = dismissState,
                        backgroundContent = {
                            Box(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .padding(horizontal = 16.dp),
                                contentAlignment = Alignment.CenterEnd,
                            ) {
                                Text(
                                    text = "Mark Read",
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.primary,
                                )
                            }
                        },
                    ) {
                        NotificationRow(
                            notification = notification,
                            isUnread = isUnread,
                            onClick = {
                                scope.launch {
                                    val markedId = notificationManager.markNotificationRead(notification.notificationId)
                                    if (markedId != null) {
                                        runCatching { apiClient.acknowledgeNotifications(listOf(markedId)) }
                                    }
                                }
                            },
                        )
                    }

                    HorizontalDivider(modifier = Modifier.padding(horizontal = 16.dp))
                }
            }
        }
    }
}
