package com.claudiator.app.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import com.claudiator.app.ui.theme.AppTheme
import com.claudiator.app.ui.theme.AppThemeDefaults
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.LocalIsDarkTheme

@Composable
fun ThemePreviewCard(
    theme: AppTheme,
    isSelected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val currentTheme = LocalAppTheme.current
    val isDark = LocalIsDarkTheme.current
    val borderColor = if (isSelected) currentTheme.uiAccent else currentTheme.cardBorder(isDark).copy(alpha = AppThemeDefaults.cardBorderOpacity)
    val borderWidth = if (isSelected) 2.dp else AppThemeDefaults.cardBorderWidth.dp

    Column(
        modifier = modifier
            .clip(RoundedCornerShape(AppThemeDefaults.cardCornerRadius.dp))
            .border(borderWidth, borderColor, RoundedCornerShape(AppThemeDefaults.cardCornerRadius.dp))
            .background(currentTheme.cardBackground(isDark))
            .clickable(onClick = onClick)
            .padding(12.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            theme.preview.forEach { color ->
                Box(
                    modifier = Modifier
                        .size(16.dp)
                        .clip(CircleShape)
                        .background(color),
                )
            }
        }
        Spacer(Modifier.height(6.dp))
        Text(
            text = theme.name,
            style = MaterialTheme.typography.labelSmall,
        )
    }
}
