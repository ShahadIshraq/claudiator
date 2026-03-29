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

    LaunchedEffect(dataVersion) {
        viewModel.refresh(apiClient)
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Devices") },
                actions = {
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
        PullToRefreshBox(
            isRefreshing = state.isLoading,
            onRefresh = { viewModel.refresh(apiClient) },
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues),
        ) {
            if (state.devices.isEmpty() && !state.isLoading) {
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
