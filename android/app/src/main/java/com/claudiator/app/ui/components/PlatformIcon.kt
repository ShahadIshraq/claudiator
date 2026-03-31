package com.claudiator.app.ui.components

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.*
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.platformColor

@Composable
fun PlatformIcon(
    platform: String,
    modifier: Modifier = Modifier,
) {
    val theme = LocalAppTheme.current
    val icon: ImageVector = when (platform.lowercase()) {
        "mac", "macos", "darwin" -> Icons.Outlined.Laptop
        "linux" -> Icons.Outlined.Computer
        "windows" -> Icons.Outlined.DesktopWindows
        else -> Icons.Outlined.Monitor
    }
    Icon(
        imageVector = icon,
        contentDescription = platform,
        tint = theme.platformColor(platform),
        modifier = modifier,
    )
}
