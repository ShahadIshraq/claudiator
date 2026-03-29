package com.claudiator.app.ui.sessions

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Notifications
import androidx.compose.material.icons.outlined.ViewList
import androidx.compose.material.icons.outlined.ViewModule
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
import com.claudiator.app.ui.components.ThemedSegmentedPicker
import com.claudiator.app.viewmodels.AllSessionsViewModel
import com.claudiator.app.viewmodels.SessionFilter

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AllSessionsScreen(
    apiClient: ApiClient,
    versionMonitor: VersionMonitor,
    notificationManager: AppNotificationManager,
    onSessionClick: (String) -> Unit,
    onDeviceClick: (String) -> Unit,
    viewModel: AllSessionsViewModel = viewModel(),
) {
    val state by viewModel.uiState.collectAsState()
    val notifState by notificationManager.state.collectAsState()
    val dataVersion by versionMonitor.dataVersion.collectAsState()
    val listState = rememberLazyListState()

    LaunchedEffect(dataVersion, state.filter) {
        viewModel.refresh(apiClient)
    }

    // Infinite scroll: trigger load more when near the end
    val shouldLoadMore by remember {
        derivedStateOf {
            val layoutInfo = listState.layoutInfo
            val totalItems = layoutInfo.totalItemsCount
            val lastVisibleIndex = layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0
            lastVisibleIndex >= totalItems - 3 && totalItems > 0
        }
    }

    LaunchedEffect(shouldLoadMore) {
        if (shouldLoadMore && state.hasMore && !state.isLoadingMore) {
            viewModel.loadMore(apiClient)
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Sessions") },
                actions = {
                    IconButton(onClick = { viewModel.toggleGrouping() }) {
                        Icon(
                            imageVector = if (state.isGroupedByDevice) Icons.Outlined.ViewList else Icons.Outlined.ViewModule,
                            contentDescription = if (state.isGroupedByDevice) "Flat list" else "Group by device",
                        )
                    }
                    BadgedBox(
                        badge = {
                            if (notifState.unreadCount > 0) {
                                Badge { Text(notifState.unreadCount.toString()) }
                            }
                        },
                        modifier = Modifier.padding(end = 8.dp),
                    ) {
                        Icon(
                            imageVector = Icons.Outlined.Notifications,
                            contentDescription = "Notifications",
                        )
                    }
                },
            )
        },
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                horizontalArrangement = Arrangement.Center,
            ) {
                ThemedSegmentedPicker(
                    options = SessionFilter.entries,
                    selected = state.filter,
                    onSelect = { viewModel.setFilter(it) },
                    label = { it.label },
                )
            }

            PullToRefreshBox(
                isRefreshing = state.isLoading,
                onRefresh = { viewModel.refresh(apiClient) },
                modifier = Modifier.fillMaxSize(),
            ) {
                if (state.sessions.isEmpty() && !state.isLoading) {
                    Box(
                        modifier = Modifier.fillMaxSize(),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = "No sessions",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                } else if (state.isGroupedByDevice) {
                    LazyColumn(
                        state = listState,
                        modifier = Modifier.fillMaxSize(),
                        contentPadding = PaddingValues(16.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        state.groupedSessions.forEach { (deviceId, sessions) ->
                            val firstSession = sessions.firstOrNull()
                            val deviceName = firstSession?.deviceName ?: deviceId
                            val platform = firstSession?.platform ?: "unknown"
                            val isExpanded = deviceId in state.expandedDevices

                            item(key = "group_$deviceId") {
                                DeviceGroupCard(
                                    deviceId = deviceId,
                                    deviceName = deviceName,
                                    platform = platform,
                                    sessions = sessions,
                                    isExpanded = isExpanded,
                                    onToggle = { viewModel.toggleDevice(deviceId) },
                                    onSessionClick = onSessionClick,
                                )
                            }
                        }

                        if (state.isLoadingMore) {
                            item {
                                Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(vertical = 8.dp),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    CircularProgressIndicator(modifier = Modifier.size(24.dp))
                                }
                            }
                        }
                    }
                } else {
                    LazyColumn(
                        state = listState,
                        modifier = Modifier.fillMaxSize(),
                    ) {
                        items(state.sessions, key = { it.sessionId }) { session ->
                            AllSessionRow(
                                session = session,
                                hasNotification = session.sessionId in notifState.unreadSessionIds,
                                onClick = { onSessionClick(session.sessionId) },
                            )
                            HorizontalDivider(
                                modifier = Modifier.padding(start = 56.dp),
                                thickness = 0.5.dp,
                            )
                        }

                        if (state.isLoadingMore) {
                            item {
                                Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(vertical = 8.dp),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    CircularProgressIndicator(modifier = Modifier.size(24.dp))
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
