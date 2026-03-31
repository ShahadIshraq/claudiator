package com.claudiator.app.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.claudiator.app.BuildConfig
import com.claudiator.app.services.ApiClient
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.ThemeManager

@Composable
fun SettingsScreen(
    apiClient: ApiClient,
    themeManager: ThemeManager,
    onDisconnect: () -> Unit,
) {
    val theme = LocalAppTheme.current
    val scrollState = rememberScrollState()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(scrollState)
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        // Logo + version header
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = 16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                text = "Claudiator",
                style = MaterialTheme.typography.headlineMedium,
                color = theme.uiAccent,
            )
            Spacer(Modifier.height(4.dp))
            Text(
                text = "v${BuildConfig.VERSION_NAME}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        AppearanceSection(themeManager = themeManager)

        ServerConfigSection(apiClient = apiClient)

        DangerZoneSection(onDisconnect = onDisconnect)
    }
}
