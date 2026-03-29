package com.claudiator.app.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue

val LocalAppTheme = compositionLocalOf<AppTheme> { StandardTheme }
val LocalIsDarkTheme = compositionLocalOf { false }

@Composable
fun ClaudiatorTheme(
    themeManager: ThemeManager,
    content: @Composable () -> Unit,
) {
    val theme by themeManager.currentTheme.collectAsState()
    val mode by themeManager.appearanceMode.collectAsState()

    val isDark = when (mode) {
        AppearanceMode.SYSTEM -> isSystemInDarkTheme()
        AppearanceMode.LIGHT -> false
        AppearanceMode.DARK -> true
    }

    MaterialTheme(
        colorScheme = if (isDark) darkColorScheme() else lightColorScheme(),
    ) {
        CompositionLocalProvider(
            LocalAppTheme provides theme,
            LocalIsDarkTheme provides isDark,
        ) {
            content()
        }
    }
}
