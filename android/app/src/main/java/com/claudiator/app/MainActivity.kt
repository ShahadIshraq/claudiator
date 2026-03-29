package com.claudiator.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import com.claudiator.app.navigation.AppNavigation
import com.claudiator.app.services.FcmService
import com.claudiator.app.ui.theme.ClaudiatorTheme
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        val app = application as ClaudiatorApp

        FcmService.getStoredToken(this)?.let { token ->
            if (app.secureStorage.isConfigured) {
                CoroutineScope(Dispatchers.IO).launch {
                    try { app.apiClient.registerPushToken(token) } catch (_: Exception) {}
                }
            }
        }

        setContent {
            ClaudiatorTheme(themeManager = app.themeManager) {
                AppNavigation(app = app)
            }
        }
    }
}
