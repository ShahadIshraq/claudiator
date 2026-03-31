package com.claudiator.app.ui.devices

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Notifications
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
import com.claudiator.app.ui.notifications.NotificationListSheet
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.LocalIsDarkTheme
import com.claudiator.app.viewmodels.DeviceListViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DeviceListScreen(
    apiClient: ApiClient,
    versionMonitor: VersionMonitor,
    notificationManager: AppNotificationManager,
    onDeviceClick: (String) -> Unit,
    viewModel: DeviceListViewModel = viewModel(),
) {
    val state by viewModel.uiState.collectAsState()
    val notifState by notificationManager.state.collectAsState()
    val dataVersion by versionMonitor.dataVersion.collectAsState()
    val theme = LocalAppTheme.current
    val isDark = LocalIsDarkTheme.current
    var showNotifications by remember { mutableStateOf(false) }

    // Auto-refresh on data version change
    LaunchedEffect(dataVersion) {
        viewModel.refresh(apiClient)
    }

    if (showNotifications) {
        NotificationListSheet(
            notificationManager = notificationManager,
            apiClient = apiClient,
            onDismiss = { showNotifications = false },
        )
    }

    Scaffold(
        containerColor = theme.pageBackground(isDark),
        topBar = {
            TopAppBar(
                title = { Text("Devices") },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = theme.pageBackground(isDark),
                ),
                actions = {
                    IconButton(onClick = { showNotifications = true }) {
                        BadgedBox(
                            badge = {
                                if (notifState.unreadCount > 0) {
                                    Badge { Text(notifState.unreadCount.toString()) }
                                }
                            },
                        ) {
                            Icon(
                                imageVector = Icons.Outlined.Notifications,
                                contentDescription = "Notifications",
                            )
                        }
                    }
                },
            )
        },
    ) { paddingValues ->
        PullToRefreshBox(
            isRefreshing = state.isRefreshing,
            onRefresh = { viewModel.refresh(apiClient) },
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
        ) {
            if (state.isLoading && state.devices.isEmpty()) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center,
                ) {
                    CircularProgressIndicator()
                }
            } else if (state.devices.isEmpty()) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = "No devices found",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    items(state.devices, key = { it.deviceId }) { device ->
                        DeviceRow(
                            device = device,
                            statusCounts = state.statusCounts[device.deviceId],
                            onClick = { onDeviceClick(device.deviceId) },
                        )
                        HorizontalDivider(
                            modifier = Modifier.padding(start = 56.dp),
                            thickness = 0.5.dp,
                        )
                    }
                }
            }
        }
    }
}
