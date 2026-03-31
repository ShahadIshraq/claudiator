package com.claudiator.app.navigation

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Devices
import androidx.compose.material.icons.outlined.*
import androidx.navigation.NavType
import androidx.navigation.navArgument
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.navigation.NavHostController
import androidx.navigation.compose.*
import com.claudiator.app.ClaudiatorApp
import com.claudiator.app.ui.devices.DeviceDetailScreen
import com.claudiator.app.ui.devices.DeviceListScreen
import com.claudiator.app.ui.sessions.AllSessionsScreen
import com.claudiator.app.ui.sessions.SessionDetailScreen
import com.claudiator.app.ui.settings.SettingsScreen
import com.claudiator.app.ui.setup.SetupScreen
import kotlinx.coroutines.launch

@Composable
fun AppNavigation(app: ClaudiatorApp) {
    val isConfigured by app.apiClient.isConfigured.collectAsState()
    val navController = rememberNavController()

    NavHost(
        navController = navController,
        startDestination = if (isConfigured) Screen.Main.route else Screen.Setup.route,
    ) {
        composable(Screen.Setup.route) {
            SetupScreen(
                apiClient = app.apiClient,
                onConnected = {
                    navController.navigate(Screen.Main.route) {
                        popUpTo(Screen.Setup.route) { inclusive = true }
                    }
                },
            )
        }
        composable(Screen.Main.route) {
            MainScaffold(app = app, rootNavController = navController)
        }
        composable(
            Screen.DeviceDetail.route,
            arguments = listOf(navArgument("deviceId") { type = NavType.StringType }),
        ) { backStackEntry ->
            val deviceId = backStackEntry.arguments?.getString("deviceId") ?: ""
            DeviceDetailScreen(
                deviceId = deviceId,
                apiClient = app.apiClient,
                versionMonitor = app.versionMonitor,
                notificationManager = app.notificationManager,
                onSessionClick = { sessionId ->
                    navController.navigate(Screen.SessionDetail.createRoute(sessionId))
                },
                onBack = { navController.popBackStack() },
            )
        }
        composable(
            Screen.SessionDetail.route,
            arguments = listOf(navArgument("sessionId") { type = NavType.StringType }),
        ) { backStackEntry ->
            val sessionId = backStackEntry.arguments?.getString("sessionId") ?: ""
            SessionDetailScreen(
                sessionId = sessionId,
                apiClient = app.apiClient,
                versionMonitor = app.versionMonitor,
                notificationManager = app.notificationManager,
                onDeviceClick = { deviceId ->
                    navController.navigate(Screen.DeviceDetail.createRoute(deviceId))
                },
                onBack = { navController.popBackStack() },
            )
        }
    }
}

@Composable
fun MainScaffold(
    app: ClaudiatorApp,
    rootNavController: NavHostController,
) {
    val pagerState = rememberPagerState(initialPage = 1) { 3 }
    val scope = rememberCoroutineScope()

    // Start polling once
    LaunchedEffect(Unit) {
        app.versionMonitor.start(this)
    }

    Scaffold(
        bottomBar = {
            NavigationBar {
                val tabs = listOf(
                    Triple(0, "Devices", Icons.Filled.Devices),
                    Triple(1, "Sessions", Icons.Outlined.Code),
                    Triple(2, "Settings", Icons.Outlined.Settings),
                )
                tabs.forEach { (index, label, icon) ->
                    NavigationBarItem(
                        icon = { Icon(icon, contentDescription = label) },
                        label = { Text(label) },
                        selected = pagerState.currentPage == index,
                        onClick = { scope.launch { pagerState.animateScrollToPage(index) } },
                    )
                }
            }
        },
    ) { padding ->
        HorizontalPager(
            state = pagerState,
            modifier = Modifier
                .fillMaxSize()
                .padding(padding),
            // Keep adjacent pages alive to avoid re-creation on swipe
            beyondViewportPageCount = 2,
        ) { page ->
            when (page) {
                0 -> DeviceListScreen(
                    apiClient = app.apiClient,
                    versionMonitor = app.versionMonitor,
                    notificationManager = app.notificationManager,
                    onDeviceClick = { deviceId ->
                        rootNavController.navigate(Screen.DeviceDetail.createRoute(deviceId))
                    },
                )
                1 -> AllSessionsScreen(
                    apiClient = app.apiClient,
                    versionMonitor = app.versionMonitor,
                    notificationManager = app.notificationManager,
                    onSessionClick = { sessionId ->
                        rootNavController.navigate(Screen.SessionDetail.createRoute(sessionId))
                    },
                    onDeviceClick = { deviceId ->
                        rootNavController.navigate(Screen.DeviceDetail.createRoute(deviceId))
                    },
                )
                2 -> SettingsScreen(
                    apiClient = app.apiClient,
                    themeManager = app.themeManager,
                    onDisconnect = {
                        app.apiClient.disconnect()
                        app.versionMonitor.stop()
                        rootNavController.navigate(Screen.Setup.route) {
                            popUpTo(Screen.Main.route) { inclusive = true }
                        }
                    },
                )
            }
        }
    }
}
