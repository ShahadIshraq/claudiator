package com.claudiator.app.ui.theme

import android.content.Context
import android.content.SharedPreferences
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

enum class AppearanceMode { SYSTEM, LIGHT, DARK }

class ThemeManager(context: Context) {
    private val prefs: SharedPreferences =
        context.getSharedPreferences("claudiator_theme", Context.MODE_PRIVATE)

    private val _currentTheme = MutableStateFlow(loadTheme())
    val currentTheme: StateFlow<AppTheme> = _currentTheme.asStateFlow()

    private val _appearanceMode = MutableStateFlow(loadAppearance())
    val appearanceMode: StateFlow<AppearanceMode> = _appearanceMode.asStateFlow()

    fun selectTheme(theme: AppTheme) {
        prefs.edit().putString("theme_id", theme.id).apply()
        _currentTheme.value = theme
    }

    fun setAppearance(mode: AppearanceMode) {
        prefs.edit().putString("appearance_mode", mode.name).apply()
        _appearanceMode.value = mode
    }

    private fun loadTheme(): AppTheme {
        val savedId = prefs.getString("theme_id", "standard")
        return allThemes.firstOrNull { it.id == savedId } ?: StandardTheme
    }

    private fun loadAppearance(): AppearanceMode {
        val saved = prefs.getString("appearance_mode", "SYSTEM")
        return try {
            AppearanceMode.valueOf(saved ?: "SYSTEM")
        } catch (_: Exception) {
            AppearanceMode.SYSTEM
        }
    }
}
