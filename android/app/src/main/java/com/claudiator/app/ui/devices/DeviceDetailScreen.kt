package com.claudiator.app.ui.devices

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material3.*
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.claudiator.app.services.ApiClient
import com.claudiator.app.services.AppNotificationManager
import com.claudiator.app.services.VersionMonitor
import com.claudiator.app.ui.components.PlatformIcon
import com.claudiator.app.ui.components.ThemedCard
import com.claudiator.app.ui.components.ThemedSegmentedPicker
import com.claudiator.app.ui.sessions.SessionRow
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.LocalIsDarkTheme
import com.claudiator.app.util.relativeTime
import com.claudiator.app.viewmodels.SessionFilter
import com.claudiator.app.viewmodels.SessionListViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DeviceDetailScreen(
    deviceId: String,
    apiClient: ApiClient,
    versionMonitor: VersionMonitor,
    notificationManager: AppNotificationManager,
    onSessionClick: (String) -> Unit,
    onBack: () -> Unit,
    viewModel: SessionListViewModel = viewModel(),
) {
    val state by viewModel.uiState.collectAsState()
    val dataVersion by versionMonitor.dataVersion.collectAsState()
    val theme = LocalAppTheme.current
    val isDark = LocalIsDarkTheme.current

    LaunchedEffect(dataVersion, state.filter) {
        viewModel.refresh(apiClient, deviceId)
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = state.sessions.firstOrNull()?.deviceName ?: deviceId,
                        maxLines = 1,
                    )
                },
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
            onRefresh = { viewModel.refresh(apiClient, deviceId) },
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                item {
                    ThemedCard {
                        val firstSession = state.sessions.firstOrNull()
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            PlatformIcon(
                                platform = firstSession?.platform ?: "unknown",
                                modifier = Modifier.size(36.dp),
                            )
                            Column {
                                Text(
                                    text = firstSession?.deviceName ?: deviceId,
                                    style = MaterialTheme.typography.titleMedium,
                                )
                                Text(
                                    text = firstSession?.platform ?: "Unknown platform",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                        Spacer(Modifier.height(12.dp))
                        InfoRow(label = "Device ID", value = deviceId)
                        InfoRow(
                            label = "Active sessions",
                            value = state.sessions.count { it.status != "ended" }.toString(),
                        )
                    }
                }

                item {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = "Sessions",
                            style = MaterialTheme.typography.titleMedium,
                        )
                        ThemedSegmentedPicker(
                            options = SessionFilter.entries,
                            selected = state.filter,
                            onSelect = { viewModel.setFilter(it) },
                            label = { it.label },
                        )
                    }
                }

                if (state.sessions.isEmpty() && !state.isLoading) {
                    item {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(vertical = 32.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(
                                text = "No sessions",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                } else {
                    items(state.sessions, key = { it.sessionId }) { session ->
                        ThemedCard {
                            SessionRow(
                                session = session,
                                onClick = { onSessionClick(session.sessionId) },
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun InfoRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 2.dp),
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
