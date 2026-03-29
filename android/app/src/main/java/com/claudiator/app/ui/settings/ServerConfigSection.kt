package com.claudiator.app.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import com.claudiator.app.services.ApiClient
import com.claudiator.app.ui.components.ThemedCard
import com.claudiator.app.ui.theme.LocalAppTheme
import kotlinx.coroutines.launch

private enum class ConnectionStatus { IDLE, TESTING, CONNECTED, FAILED }

@Composable
fun ServerConfigSection(apiClient: ApiClient) {
    val theme = LocalAppTheme.current
    val scope = rememberCoroutineScope()

    var serverUrl by remember { mutableStateOf("") }
    var apiKey by remember { mutableStateOf("") }
    var passwordVisible by remember { mutableStateOf(false) }
    var connectionStatus by remember { mutableStateOf(ConnectionStatus.IDLE) }
    var statusMessage by remember { mutableStateOf<String?>(null) }

    ThemedCard {
        Text(
            text = "Server Configuration",
            style = MaterialTheme.typography.titleSmall,
            modifier = Modifier.padding(bottom = 12.dp),
        )

        OutlinedTextField(
            value = serverUrl,
            onValueChange = { serverUrl = it },
            label = { Text("Server URL") },
            placeholder = { Text("https://your-server.com") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri),
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(Modifier.height(12.dp))

        OutlinedTextField(
            value = apiKey,
            onValueChange = { apiKey = it },
            label = { Text("API Key") },
            singleLine = true,
            visualTransformation = if (passwordVisible) VisualTransformation.None else PasswordVisualTransformation(),
            trailingIcon = {
                IconButton(onClick = { passwordVisible = !passwordVisible }) {
                    Icon(
                        imageVector = if (passwordVisible) Icons.Default.VisibilityOff else Icons.Default.Visibility,
                        contentDescription = if (passwordVisible) "Hide" else "Show",
                    )
                }
            },
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(Modifier.height(12.dp))

        // Connection status indicator
        Row(verticalAlignment = Alignment.CenterVertically) {
            val dotColor = when (connectionStatus) {
                ConnectionStatus.CONNECTED -> Color(0xFF4CAF50)
                ConnectionStatus.FAILED -> Color(0xFFF44336)
                ConnectionStatus.TESTING -> Color(0xFFFF9800)
                ConnectionStatus.IDLE -> Color.Gray
            }
            Box(
                modifier = Modifier
                    .size(10.dp)
                    .clip(CircleShape)
                    .background(dotColor),
            )
            Spacer(Modifier.width(8.dp))
            Text(
                text = statusMessage ?: when (connectionStatus) {
                    ConnectionStatus.CONNECTED -> "Connected"
                    ConnectionStatus.FAILED -> "Connection failed"
                    ConnectionStatus.TESTING -> "Testing..."
                    ConnectionStatus.IDLE -> "Not tested"
                },
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        Spacer(Modifier.height(12.dp))

        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            OutlinedButton(
                onClick = {
                    scope.launch {
                        connectionStatus = ConnectionStatus.TESTING
                        statusMessage = null
                        runCatching { apiClient.ping() }
                            .onSuccess {
                                connectionStatus = ConnectionStatus.CONNECTED
                                statusMessage = "Connected"
                            }
                            .onFailure { e ->
                                connectionStatus = ConnectionStatus.FAILED
                                statusMessage = e.message ?: "Connection failed"
                            }
                    }
                },
                enabled = connectionStatus != ConnectionStatus.TESTING,
                modifier = Modifier.weight(1f),
            ) {
                if (connectionStatus == ConnectionStatus.TESTING) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(16.dp),
                        strokeWidth = 2.dp,
                    )
                    Spacer(Modifier.width(8.dp))
                }
                Text("Test Connection")
            }

            Button(
                onClick = {
                    if (serverUrl.isNotBlank() && apiKey.isNotBlank()) {
                        apiClient.configure(serverUrl.trim(), apiKey.trim())
                        statusMessage = "Saved"
                    }
                },
                enabled = serverUrl.isNotBlank() && apiKey.isNotBlank(),
                modifier = Modifier.weight(1f),
            ) {
                Text("Save")
            }
        }
    }
}
