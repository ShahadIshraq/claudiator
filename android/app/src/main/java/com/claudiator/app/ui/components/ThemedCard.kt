package com.claudiator.app.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.claudiator.app.ui.theme.AppThemeDefaults
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.LocalIsDarkTheme

@Composable
fun ThemedCard(
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    val theme = LocalAppTheme.current
    val isDark = LocalIsDarkTheme.current
    Card(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(AppThemeDefaults.cardCornerRadius.dp),
        colors = CardDefaults.cardColors(containerColor = theme.cardBackground(isDark)),
        border = BorderStroke(
            AppThemeDefaults.cardBorderWidth.dp,
            theme.cardBorder(isDark).copy(alpha = AppThemeDefaults.cardBorderOpacity),
        ),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp),
    ) {
        Column(modifier = Modifier.padding(16.dp), content = content)
    }
}
