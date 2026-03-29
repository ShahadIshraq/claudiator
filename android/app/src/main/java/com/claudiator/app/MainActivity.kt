package com.claudiator.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import com.claudiator.app.navigation.AppNavigation
import com.claudiator.app.ui.theme.ClaudiatorTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        val app = application as ClaudiatorApp

        setContent {
            ClaudiatorTheme(themeManager = app.themeManager) {
                AppNavigation(app = app)
            }
        }
    }
}
