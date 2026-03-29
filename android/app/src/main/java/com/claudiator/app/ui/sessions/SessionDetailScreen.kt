package com.claudiator.app.ui.sessions

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material3.*
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.claudiator.app.services.ApiClient
import com.claudiator.app.services.AppNotificationManager
import com.claudiator.app.services.VersionMonitor
import com.claudiator.app.ui.components.PlatformIcon
import com.claudiator.app.ui.components.ThemedCard
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.statusColor
import com.claudiator.app.util.cwdShortDisplay
import com.claudiator.app.util.relativeTime
import com.claudiator.app.util.statusDisplayLabel
import com.claudiator.app.viewmodels.EventListViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SessionDetailScreen(
    sessionId: String,
    apiClient: ApiClient,
    versionMonitor: VersionMonitor,
    notificationManager: AppNotificationManager,
    onDeviceClick: (String) -> Unit,
    onBack: () -> Unit,
    viewModel: EventListViewModel = viewModel(),
) {
    val state by viewModel.uiState.collectAsState()
    val dataVersion by versionMonitor.dataVersion.collectAsState()
    var detailsExpanded by remember { mutableStateOf(false) }

    // Mark session as read when entering
    LaunchedEffect(sessionId) {
        val markedIds = notificationManager.markSessionRead(sessionId)
        if (markedIds.isNotEmpty()) {
            try {
                apiClient.acknowledgeNotifications(markedIds)
            } catch (_: Exception) {
                // best-effort
            }
        }
    }

    LaunchedEffect(dataVersion) {
        viewModel.refresh(apiClient, sessionId)
    }

    // Derive session info from first event's context — use events as source of truth for timing
    // We don't have a separate session detail API so derive from event list state
    val firstEvent = state.events.firstOrNull()
    val lastEvent = state.events.lastOrNull()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Session", maxLines = 1) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Outlined.ArrowBack,
                            contentDescription = "Back",
                        )
                    }
                },
            )
        },
    ) { paddingValues ->
        PullToRefreshBox(
            isRefreshing = state.isLoading,
            onRefresh = { viewModel.refresh(apiClient, sessionId) },
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                // Status header
                item {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 12.dp),
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(24.dp)
                                    .clip(CircleShape)
                                    .background(MaterialTheme.colorScheme.surfaceVariant),
                            )
                            Column {
                                Text(
                                    text = sessionId.take(12),
                                    style = MaterialTheme.typography.titleSmall,
                                    fontWeight = FontWeight.SemiBold,
                                )
                            }
                        }
                    }
                }

                // Collapsible details section
                item {
                    ThemedCard(modifier = Modifier.padding(horizontal = 16.dp)) {
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable { detailsExpanded = !detailsExpanded }
                                .padding(vertical = 4.dp),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                text = "Details",
                                style = MaterialTheme.typography.titleSmall,
                                fontWeight = FontWeight.Medium,
                            )
                            Icon(
                                imageVector = if (detailsExpanded) Icons.Default.KeyboardArrowUp else Icons.Default.KeyboardArrowDown,
                                contentDescription = if (detailsExpanded) "Collapse" else "Expand",
                                modifier = Modifier.size(20.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }

                        AnimatedVisibility(
                            visible = detailsExpanded,
                            enter = expandVertically(),
                            exit = shrinkVertically(),
                        ) {
                            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                                HorizontalDivider(thickness = 0.5.dp)
                                Spacer(Modifier.height(4.dp))
                                DetailRow(label = "Session ID", value = sessionId)
                                if (firstEvent != null) {
                                    DetailRow(
                                        label = "First event",
                                        value = relativeTime(firstEvent.timestamp),
                                    )
                                }
                                if (lastEvent != null) {
                                    DetailRow(
                                        label = "Last event",
                                        value = relativeTime(lastEvent.timestamp),
                                    )
                                }
                                DetailRow(
                                    label = "Event count",
                                    value = state.events.size.toString(),
                                )
                            }
                        }
                    }
                }

                item { Spacer(Modifier.height(12.dp)) }

                // Events header
                item {
                    Text(
                        text = "Events",
                        style = MaterialTheme.typography.titleMedium,
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp),
                    )
                }

                if (state.events.isEmpty() && !state.isLoading) {
                    item {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(vertical = 32.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(
                                text = "No events",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                } else {
                    items(state.events, key = { it.id }) { event ->
                        EventRow(event = event)
                        HorizontalDivider(
                            modifier = Modifier.padding(start = 36.dp),
                            thickness = 0.5.dp,
                        )
                    }
                }

                item { Spacer(Modifier.height(16.dp)) }
            }
        }
    }
}

@Composable
private fun DetailRow(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodySmall,
        )
    }
}
