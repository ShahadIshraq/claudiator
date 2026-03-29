package com.claudiator.app.navigation

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.navigation.NavHostController
import androidx.navigation.compose.*
import com.claudiator.app.ClaudiatorApp
import com.claudiator.app.ui.theme.LocalAppTheme
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
            // Placeholder - replaced by Task 11
            PlaceholderScreen("Setup")
        }
        composable(Screen.Main.route) {
            MainScaffold(app = app, rootNavController = navController)
        }
        composable(
            Screen.DeviceDetail.route,
            arguments = listOf(navArgument("deviceId") { defaultValue = "" }),
        ) { backStackEntry ->
            val deviceId = backStackEntry.arguments?.getString("deviceId") ?: ""
            // Placeholder - replaced by Task 12
            PlaceholderScreen("Device: $deviceId")
        }
        composable(
            Screen.SessionDetail.route,
            arguments = listOf(navArgument("sessionId") { defaultValue = "" }),
        ) { backStackEntry ->
            val sessionId = backStackEntry.arguments?.getString("sessionId") ?: ""
            // Placeholder - replaced by Task 13
            PlaceholderScreen("Session: $sessionId")
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

    LaunchedEffect(Unit) {
        app.versionMonitor.start(this)
    }

    Scaffold(
        bottomBar = {
            NavigationBar {
                val tabs = listOf(
                    Triple(0, "Devices", Icons.Outlined.Devices),
                    Triple(1, "Sessions", Icons.Outlined.Terminal),
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
        ) { page ->
            when (page) {
                0 -> PlaceholderScreen("Devices") // Replaced by Task 12
                1 -> PlaceholderScreen("Sessions") // Replaced by Task 13
                2 -> PlaceholderScreen("Settings") // Replaced by Task 15
            }
        }
    }
}

@Composable
private fun PlaceholderScreen(name: String) {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Text(name, style = MaterialTheme.typography.headlineMedium)
    }
}
