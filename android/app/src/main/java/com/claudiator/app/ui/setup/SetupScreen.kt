package com.claudiator.app.ui.setup

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.claudiator.app.services.ApiClient
import com.claudiator.app.ui.theme.LocalAppTheme
import com.claudiator.app.ui.theme.LocalIsDarkTheme
import com.claudiator.app.viewmodels.SetupViewModel

@Composable
fun SetupScreen(
    apiClient: ApiClient,
    onConnected: () -> Unit,
    viewModel: SetupViewModel = viewModel(),
) {
    val state by viewModel.uiState.collectAsState()
    val theme = LocalAppTheme.current
    val isDark = LocalIsDarkTheme.current
    var passwordVisible by remember { mutableStateOf(false) }

    LaunchedEffect(state.connectionSuccess) {
        if (state.connectionSuccess) onConnected()
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(theme.pageBackground(isDark))
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Text(
            text = "Claudiator",
            style = MaterialTheme.typography.headlineLarge,
            color = theme.uiAccent,
        )

        Spacer(Modifier.height(8.dp))

        Text(
            text = "Connect to your Claudiator server",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        Spacer(Modifier.height(32.dp))

        OutlinedTextField(
            value = state.serverUrl,
            onValueChange = viewModel::updateServerUrl,
            label = { Text("Server URL") },
            placeholder = { Text("https://your-server.com") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri),
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(Modifier.height(16.dp))

        OutlinedTextField(
            value = state.apiKey,
            onValueChange = viewModel::updateApiKey,
            label = { Text("API Key") },
            singleLine = true,
            visualTransformation = if (passwordVisible) VisualTransformation.None else PasswordVisualTransformation(),
            trailingIcon = {
                IconButton(onClick = { passwordVisible = !passwordVisible }) {
                    Icon(
                        if (passwordVisible) Icons.Default.VisibilityOff else Icons.Default.Visibility,
                        contentDescription = if (passwordVisible) "Hide" else "Show",
                    )
                }
            },
            modifier = Modifier.fillMaxWidth(),
        )

        if (state.error != null) {
            Spacer(Modifier.height(16.dp))
            Text(
                text = state.error!!,
                color = theme.uiError,
                style = MaterialTheme.typography.bodySmall,
            )
        }

        Spacer(Modifier.height(24.dp))

        Button(
            onClick = { viewModel.connect(apiClient) },
            enabled = !state.isLoading && state.serverUrl.isNotBlank() && state.apiKey.isNotBlank(),
            modifier = Modifier.fillMaxWidth(),
        ) {
            if (state.isLoading) {
                CircularProgressIndicator(
                    modifier = Modifier.size(20.dp),
                    strokeWidth = 2.dp,
                    color = MaterialTheme.colorScheme.onPrimary,
                )
                Spacer(Modifier.width(8.dp))
            }
            Text("Connect")
        }
    }
}
