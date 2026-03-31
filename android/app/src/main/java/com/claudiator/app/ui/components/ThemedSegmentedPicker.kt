package com.claudiator.app.ui.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.claudiator.app.ui.theme.AppThemeDefaults
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.LocalIsDarkTheme

@Composable
fun <T> ThemedSegmentedPicker(
    options: List<T>,
    selected: T,
    onSelect: (T) -> Unit,
    label: (T) -> String,
    modifier: Modifier = Modifier,
) {
    val theme = LocalAppTheme.current
    val isDark = LocalIsDarkTheme.current
    Row(
        modifier = modifier
            .widthIn(max = 200.dp)
            .clip(CircleShape)
            .background(theme.cardBackground(isDark).copy(alpha = 0.5f))
            .padding(3.dp),
        horizontalArrangement = Arrangement.Center,
    ) {
        options.forEach { option ->
            val isSelected = option == selected
            val bg by animateColorAsState(
                if (isSelected) theme.cardBackground(isDark) else Color.Transparent,
                label = "segBg",
            )
            Box(
                modifier = Modifier
                    .weight(1f)
                    .clip(CircleShape)
                    .then(if (isSelected) Modifier.shadow(2.dp, CircleShape) else Modifier)
                    .background(bg)
                    .clickable { onSelect(option) }
                    .padding(vertical = 8.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = label(option),
                    style = MaterialTheme.typography.labelMedium,
                    fontWeight = FontWeight.Medium,
                )
            }
        }
    }
}
