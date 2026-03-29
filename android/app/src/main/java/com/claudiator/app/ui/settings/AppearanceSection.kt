package com.claudiator.app.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.claudiator.app.ui.components.ThemePreviewCard
import com.claudiator.app.ui.components.ThemedCard
import com.claudiator.app.ui.components.ThemedSegmentedPicker
import com.claudiator.app.ui.theme.AppearanceMode
import com.claudiator.app.ui.theme.ThemeManager
import com.claudiator.app.ui.theme.allThemes

@Composable
fun AppearanceSection(themeManager: ThemeManager) {
    val currentTheme by themeManager.currentTheme.collectAsState()
    val appearanceMode by themeManager.appearanceMode.collectAsState()

    ThemedCard {
        Text(
            text = "Appearance",
            style = MaterialTheme.typography.titleSmall,
            modifier = Modifier.padding(bottom = 12.dp),
        )

        Text(
            text = "Mode",
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(bottom = 8.dp),
        )

        ThemedSegmentedPicker(
            options = AppearanceMode.entries,
            selected = appearanceMode,
            onSelect = { themeManager.setAppearance(it) },
            label = { mode ->
                when (mode) {
                    AppearanceMode.SYSTEM -> "System"
                    AppearanceMode.LIGHT -> "Light"
                    AppearanceMode.DARK -> "Dark"
                }
            },
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(Modifier.height(16.dp))

        Text(
            text = "Theme",
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(bottom = 8.dp),
        )

        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            items(allThemes) { theme ->
                ThemePreviewCard(
                    theme = theme,
                    isSelected = theme.id == currentTheme.id,
                    onClick = { themeManager.selectTheme(theme) },
                )
            }
        }
    }
}
