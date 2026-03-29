package com.claudiator.app.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.claudiator.app.ui.components.ThemedCard
import com.claudiator.app.ui.theme.LocalAppTheme

@Composable
fun DangerZoneSection(onDisconnect: () -> Unit) {
    val theme = LocalAppTheme.current
    var showDialog by remember { mutableStateOf(false) }

    ThemedCard {
        Text(
            text = "Danger Zone",
            style = MaterialTheme.typography.titleSmall,
            color = theme.uiError,
            modifier = Modifier.padding(bottom = 12.dp),
        )

        Button(
            onClick = { showDialog = true },
            colors = ButtonDefaults.buttonColors(
                containerColor = theme.uiError,
            ),
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Disconnect")
        }
    }

    if (showDialog) {
        AlertDialog(
            onDismissRequest = { showDialog = false },
            title = { Text("Disconnect?") },
            text = {
                Text("This will remove the server configuration and API key from this device. You will need to reconnect to use the app.")
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        showDialog = false
                        onDisconnect()
                    },
                ) {
                    Text("Disconnect", color = theme.uiError)
                }
            },
            dismissButton = {
                TextButton(onClick = { showDialog = false }) {
                    Text("Cancel")
                }
            },
        )
    }
}
