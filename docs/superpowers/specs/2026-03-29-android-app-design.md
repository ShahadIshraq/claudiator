# Claudiator Android App — Design Spec

## Overview

A Kotlin + Jetpack Compose Android app that is a complete 1:1 feature port of the existing Claudiator iOS app. The app monitors Claude Code sessions across devices, displays event timelines, and delivers real-time push notifications via Firebase Cloud Messaging (FCM). Includes corresponding server-side FCM integration in the existing Rust server.

**Target:** Android 8.0+ (API 26)
**Architecture:** Single-Activity MVVM with Compose Navigation
**Dependencies:** Minimal — Ktor Client, Firebase Messaging, AndroidX libraries

---

## 1. Project Structure & Build Configuration

### Repository Layout

```
android/
├── app/
│   ├── src/
│   │   ├── main/
│   │   │   ├── java/com/claudiator/app/
│   │   │   │   ├── ClaudiatorApp.kt
│   │   │   │   ├── MainActivity.kt
│   │   │   │   ├── navigation/
│   │   │   │   │   ├── AppNavigation.kt
│   │   │   │   │   ├── Screen.kt
│   │   │   │   │   └── MainTabs.kt
│   │   │   │   ├── models/
│   │   │   │   │   ├── Device.kt
│   │   │   │   │   ├── Session.kt
│   │   │   │   │   ├── Event.kt
│   │   │   │   │   ├── AppNotification.kt
│   │   │   │   │   └── ApiResponses.kt
│   │   │   │   ├── services/
│   │   │   │   │   ├── ApiClient.kt
│   │   │   │   │   ├── SecureStorage.kt
│   │   │   │   │   ├── AppNotificationManager.kt
│   │   │   │   │   ├── VersionMonitor.kt
│   │   │   │   │   └── FcmService.kt
│   │   │   │   ├── viewmodels/
│   │   │   │   │   ├── SetupViewModel.kt
│   │   │   │   │   ├── DeviceListViewModel.kt
│   │   │   │   │   ├── SessionListViewModel.kt
│   │   │   │   │   ├── AllSessionsViewModel.kt
│   │   │   │   │   └── EventListViewModel.kt
│   │   │   │   ├── ui/
│   │   │   │   │   ├── theme/
│   │   │   │   │   │   ├── AppTheme.kt
│   │   │   │   │   │   ├── Themes.kt
│   │   │   │   │   │   ├── ThemeManager.kt
│   │   │   │   │   │   └── Color.kt
│   │   │   │   │   ├── setup/
│   │   │   │   │   │   └── SetupScreen.kt
│   │   │   │   │   ├── devices/
│   │   │   │   │   │   ├── DeviceListScreen.kt
│   │   │   │   │   │   ├── DeviceDetailScreen.kt
│   │   │   │   │   │   └── components/
│   │   │   │   │   │       ├── DeviceRow.kt
│   │   │   │   │   │       └── StatusBadge.kt
│   │   │   │   │   ├── sessions/
│   │   │   │   │   │   ├── AllSessionsScreen.kt
│   │   │   │   │   │   ├── SessionDetailScreen.kt
│   │   │   │   │   │   └── components/
│   │   │   │   │   │       ├── SessionRow.kt
│   │   │   │   │   │       ├── AllSessionRow.kt
│   │   │   │   │   │       ├── DeviceGroupHeader.kt
│   │   │   │   │   │       ├── DeviceGroupCard.kt
│   │   │   │   │   │       └── EventRow.kt
│   │   │   │   │   ├── notifications/
│   │   │   │   │   │   ├── NotificationListScreen.kt
│   │   │   │   │   │   └── components/
│   │   │   │   │   │       └── NotificationRow.kt
│   │   │   │   │   ├── settings/
│   │   │   │   │   │   ├── SettingsScreen.kt
│   │   │   │   │   │   └── components/
│   │   │   │   │   │       ├── AppearanceSection.kt
│   │   │   │   │   │       ├── ServerConfigSection.kt
│   │   │   │   │   │       └── DangerZoneSection.kt
│   │   │   │   │   └── components/
│   │   │   │   │       ├── ThemedCard.kt
│   │   │   │   │       ├── ThemedPage.kt
│   │   │   │   │       ├── PlatformIcon.kt
│   │   │   │   │       ├── ThemedSegmentedPicker.kt
│   │   │   │   │       └── ThemePreviewCard.kt
│   │   │   │   └── util/
│   │   │   │       ├── URLValidator.kt
│   │   │   │       ├── TimeFormatting.kt
│   │   │   │       └── StringExtensions.kt
│   │   │   ├── res/
│   │   │   │   ├── drawable/          # Vector icons
│   │   │   │   ├── values/
│   │   │   │   │   ├── strings.xml
│   │   │   │   │   └── themes.xml     # Splash/system theme only
│   │   │   │   └── mipmap-*/          # App icon
│   │   │   └── AndroidManifest.xml
│   │   └── test/
│   │       └── java/com/claudiator/app/
│   │           ├── HelperTests.kt
│   │           ├── ModelDecodingTests.kt
│   │           ├── NotificationManagerTests.kt
│   │           ├── SetupViewModelTests.kt
│   │           ├── ThemeTests.kt
│   │           ├── URLValidatorTests.kt
│   │           └── ViewModelTests.kt
│   ├── build.gradle.kts
│   └── proguard-rules.pro
├── build.gradle.kts                   # Root build script
├── settings.gradle.kts
├── gradle.properties
├── gradle/wrapper/
│   ├── gradle-wrapper.jar
│   └── gradle-wrapper.properties
├── gradlew
├── gradlew.bat
├── .gitignore
└── README.md
```

### Build Configuration

**Root `build.gradle.kts`:**
- Kotlin 2.0+ with Compose compiler plugin
- Android Gradle Plugin 8.5+
- Google Services plugin for FCM
- No version catalogs (keep it simple, define versions inline)

**App `build.gradle.kts`:**
```
- compileSdk: 35
- minSdk: 26
- targetSdk: 35
- Compose BOM for version alignment
- Kotlin serialization for JSON
```

**Dependencies (exhaustive list):**
```
// Core Android
androidx.core:core-ktx
androidx.lifecycle:lifecycle-runtime-ktx
androidx.lifecycle:lifecycle-viewmodel-compose
androidx.activity:activity-compose

// Compose
androidx.compose:compose-bom (latest stable)
  - material3
  - ui
  - ui-tooling-preview
  - navigation-compose
  - material3-window-size-class (adaptive layouts)

// Networking
io.ktor:ktor-client-android
io.ktor:ktor-client-content-negotiation
io.ktor:ktor-serialization-kotlinx-json

// Secure Storage
androidx.security:security-crypto (EncryptedSharedPreferences)

// Preferences
androidx.datastore:datastore-preferences

// Push Notifications
com.google.firebase:firebase-messaging-ktx (via Firebase BOM)

// JSON
org.jetbrains.kotlinx:kotlinx-serialization-json

// Testing
junit
kotlinx-coroutines-test
```

**No other dependencies.** No Hilt/Dagger (manual DI via Application class), no Room (no local DB needed), no image loading libraries.

### .gitignore (android/)

```
*.iml
.gradle/
/local.properties
.idea/
/build/
/app/build/
/app/google-services.json
*.apk
*.aab
*.jks
*.keystore
/app/release/
```

---

## 2. Architecture

### Pattern: Single-Activity MVVM

```
┌─────────────────────────────────────────────────────┐
│  MainActivity (single Activity, Compose host)       │
│  ┌───────────────────────────────────────────────┐  │
│  │  AppNavigation (NavHost)                      │  │
│  │  ┌─────────┐ ┌──────────┐ ┌────────────────┐ │  │
│  │  │ Setup   │ │ Main     │ │ Detail screens │ │  │
│  │  │ Screen  │ │ Tabs     │ │ (push onto     │ │  │
│  │  │         │ │ (3 tabs) │ │  nav stack)    │ │  │
│  │  └─────────┘ └──────────┘ └────────────────┘ │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘

Screens ──observe──▶ ViewModels ──call──▶ Services
                     (StateFlow)          (ApiClient, etc.)
```

### Dependency Graph

```
ClaudiatorApp (Application)
  ├── creates: ApiClient (singleton)
  ├── creates: SecureStorage (singleton)
  ├── creates: AppNotificationManager (singleton)
  ├── creates: VersionMonitor (singleton)
  └── creates: ThemeManager (singleton)

MainActivity
  └── hosts: AppNavigation (Compose)
        ├── SetupScreen ← SetupViewModel(ApiClient, SecureStorage)
        ├── MainTabs
        │   ├── DeviceListScreen ← DeviceListViewModel(ApiClient)
        │   ├── AllSessionsScreen ← AllSessionsViewModel(ApiClient, AppNotificationManager)
        │   └── SettingsScreen ← ThemeManager, ApiClient, SecureStorage
        ├── DeviceDetailScreen ← SessionListViewModel(ApiClient)
        ├── SessionDetailScreen ← EventListViewModel(ApiClient), AppNotificationManager
        └── NotificationListScreen (ModalBottomSheet) ← AppNotificationManager

FcmService (runs independently)
  └── uses: ApiClient (register token), AppNotificationManager (handle message)
```

### Manual Dependency Injection

Services are created in `ClaudiatorApp` (the `Application` subclass) and accessed via a simple service locator pattern:

```kotlin
class ClaudiatorApp : Application() {
    lateinit var secureStorage: SecureStorage
    lateinit var apiClient: ApiClient
    lateinit var notificationManager: AppNotificationManager
    lateinit var versionMonitor: VersionMonitor
    lateinit var themeManager: ThemeManager

    override fun onCreate() {
        super.onCreate()
        secureStorage = SecureStorage(this)
        apiClient = ApiClient(secureStorage)
        notificationManager = AppNotificationManager(apiClient)
        versionMonitor = VersionMonitor(apiClient, notificationManager)
        themeManager = ThemeManager(this)
    }
}
```

ViewModels receive services via `ViewModelFactory` or by accessing `Application` context. This avoids adding Hilt/Dagger while keeping testability (services can be replaced with fakes in tests).

### State Management

All ViewModels expose state via `StateFlow`:

```kotlin
class DeviceListViewModel(private val apiClient: ApiClient) : ViewModel() {
    private val _uiState = MutableStateFlow(DeviceListUiState())
    val uiState: StateFlow<DeviceListUiState> = _uiState.asStateFlow()

    fun refresh() {
        viewModelScope.launch { ... }
    }
}

data class DeviceListUiState(
    val devices: List<Device> = emptyList(),
    val statusCounts: Map<String, SessionStatusCounts> = emptyMap(),
    val isLoading: Boolean = false,
    val error: String? = null
)
```

Compose screens collect state via `collectAsStateWithLifecycle()` — this is lifecycle-aware and stops collection when the screen isn't visible.

---

## 3. Data Models

All models use `kotlinx.serialization` with `@Serializable` and `@SerialName` for snake_case JSON mapping. Exact mirror of iOS models.

### Device

```kotlin
@Serializable
data class Device(
    @SerialName("device_id") val deviceId: String,
    @SerialName("device_name") val deviceName: String,
    val platform: String,
    @SerialName("first_seen") val firstSeen: String,
    @SerialName("last_seen") val lastSeen: String,
    @SerialName("active_sessions") val activeSessions: Int
)
```

### Session

```kotlin
@Serializable
data class Session(
    @SerialName("session_id") val sessionId: String,
    @SerialName("device_id") val deviceId: String,
    @SerialName("started_at") val startedAt: String,
    @SerialName("last_event") val lastEvent: String,
    val status: String,
    val cwd: String? = null,
    val title: String? = null,
    @SerialName("device_name") val deviceName: String? = null,
    val platform: String? = null
)
```

### Event

```kotlin
@Serializable
data class Event(
    val id: Int,
    @SerialName("hook_event_name") val hookEventName: String,
    val timestamp: String,
    @SerialName("tool_name") val toolName: String? = null,
    @SerialName("notification_type") val notificationType: String? = null,
    val message: String? = null
)
```

### AppNotification

```kotlin
@Serializable
data class AppNotification(
    @SerialName("id") val notificationId: String,
    @SerialName("session_id") val sessionId: String,
    @SerialName("device_id") val deviceId: String,
    val title: String,
    val body: String,
    @SerialName("notification_type") val notificationType: String,
    @SerialName("payload_json") val payloadJson: String? = null,
    @SerialName("created_at") val createdAt: String,
    val acknowledged: Boolean
)
```

### API Response Wrappers

```kotlin
@Serializable
data class PingResponse(
    val status: String,
    @SerialName("server_version") val serverVersion: String,
    @SerialName("data_version") val dataVersion: Long,
    @SerialName("notification_version") val notificationVersion: Long
)

@Serializable
data class DevicesResponse(val devices: List<Device>)

@Serializable
data class SessionsResponse(val sessions: List<Session>)

@Serializable
data class SessionListPage(
    val sessions: List<Session>,
    @SerialName("has_more") val hasMore: Boolean,
    @SerialName("next_offset") val nextOffset: Int?
)

@Serializable
data class EventsResponse(val events: List<Event>)

@Serializable
data class NotificationsResponse(val notifications: List<AppNotification>)
```

---

## 4. Services

### ApiClient

Mirrors the iOS `APIClient` exactly. Uses Ktor Client with kotlinx.serialization.

**Configuration:**
- 15s connect timeout, 30s request timeout (matching iOS)
- JSON content negotiation with `ignoreUnknownKeys = true`, snake_case naming
- Bearer token from SecureStorage
- Base URL from SecureStorage

**Retry Logic:**
- 3 attempts for network errors
- Exponential backoff: 1s, 2s, 4s
- No retry on 4xx responses

**Error Handling:**
```kotlin
sealed class ApiError : Exception() {
    object NotConfigured : ApiError()
    object InvalidURL : ApiError()
    object Unauthorized : ApiError()
    data class ServerError(val code: Int) : ApiError()
    data class NetworkError(val cause: Throwable) : ApiError()
    data class DecodingError(val cause: Throwable) : ApiError()
}
```

**Methods (1:1 with iOS):**
```kotlin
suspend fun ping(): PingResponse
suspend fun fetchDevices(): List<Device>
suspend fun fetchSessions(deviceId: String? = null, status: String? = null, limit: Int = 50): List<Session>
suspend fun fetchAllSessionsPage(excludeEnded: Boolean, limit: Int, offset: Int): SessionListPage
suspend fun fetchEvents(sessionId: String, limit: Int = 100): List<Event>
suspend fun fetchNotifications(after: String? = null, limit: Int = 50): List<AppNotification>
suspend fun acknowledgeNotifications(ids: List<String>)
suspend fun registerPushToken(token: String)
```

The `registerPushToken` method sends `platform: "android"` and `sandbox: false` (FCM doesn't have sandbox/production distinction like APNs).

**`isConfigured` property:** Checks that both baseURL and apiKey are present in SecureStorage. Exposed as a `StateFlow<Boolean>` so the UI can react to configuration changes.

### SecureStorage

Wraps `EncryptedSharedPreferences` from AndroidX Security library.

```kotlin
class SecureStorage(context: Context) {
    private val prefs: SharedPreferences = EncryptedSharedPreferences.create(
        "claudiator_secure_prefs",
        MasterKeys.getOrCreate(MasterKeys.AES256_GCM_SPEC),
        context,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
    )

    var serverUrl: String?
        get() = prefs.getString("server_url", null)
        set(value) = prefs.edit().putString("server_url", value).apply()

    var apiKey: String?
        get() = prefs.getString("api_key", null)
        set(value) = prefs.edit().putString("api_key", value).apply()

    fun clear() { prefs.edit().clear().apply() }
}
```

### AppNotificationManager

Mirrors iOS `NotificationManager`. Manages in-app notification state.

**State:**
```kotlin
data class NotificationState(
    val notifications: List<AppNotification> = emptyList(),
    val unreadCount: Int = 0,
    val unreadSessionIds: Set<String> = emptySet()
)
```

**Key behavior (matching iOS):**
- Tracks unread/read notifications via `StateFlow<NotificationState>`
- Maintains last-seen notification ID for incremental polling
- Push-received ID deduplication with 10-minute retention window
- Caps in-memory notifications at 100
- Read IDs set capped at 500
- Methods: `fetchNewNotifications()`, `markSessionRead(sessionId)`, `markNotificationRead(id)`, `markAllRead()`
- Server acknowledgement via `ApiClient.acknowledgeNotifications()`
- Fires Android system notifications for unread items not already received via FCM push

**Android System Notifications:**
- Creates a notification channel "Claudiator Sessions" on init (required API 26+)
- Uses `NotificationCompat.Builder` for system notification display
- Includes `session_id`, `device_id`, `notification_id` in intent extras for deep linking

### VersionMonitor

Mirrors iOS `VersionMonitor`. Polls the server every 10 seconds.

```kotlin
class VersionMonitor(
    private val apiClient: ApiClient,
    private val notificationManager: AppNotificationManager
) {
    private val _dataVersion = MutableStateFlow(0L)
    val dataVersion: StateFlow<Long> = _dataVersion.asStateFlow()

    private var notificationVersion = 0L
    private var pollingJob: Job? = null

    fun start(scope: CoroutineScope) {
        pollingJob?.cancel()
        pollingJob = scope.launch {
            while (isActive) {
                try {
                    val ping = apiClient.ping()
                    _dataVersion.value = ping.dataVersion
                    if (ping.notificationVersion != notificationVersion) {
                        notificationVersion = ping.notificationVersion
                        notificationManager.fetchNewNotifications()
                    }
                } catch (_: Exception) { /* silently retry next cycle */ }
                delay(10_000)
            }
        }
    }

    fun stop() { pollingJob?.cancel() }
}
```

### FcmService

Extends `FirebaseMessagingService`. Handles two events:

**`onNewToken(token)`:**
- Stores token locally
- If ApiClient is configured, registers with server via `POST /api/v1/push/register`
- If not configured yet, token is stored and registered on first successful setup

**`onMessageReceived(message)`:**
- Extracts `notification_id`, `session_id`, `device_id`, `title`, `body` from data payload
- Records the notification ID as push-received (for deduplication)
- Shows Android system notification with appropriate icon and channel
- Deep link intent points to SessionDetailScreen

---

## 5. Navigation

### Screen Routes

```kotlin
sealed class Screen(val route: String) {
    object Setup : Screen("setup")
    object Main : Screen("main")
    object DeviceDetail : Screen("device/{deviceId}") {
        fun createRoute(deviceId: String) = "device/$deviceId"
    }
    object SessionDetail : Screen("session/{sessionId}") {
        fun createRoute(sessionId: String) = "session/$sessionId"
    }
}
```

### Navigation Flow

```
App Launch
  │
  ├── ApiClient.isConfigured == false ──▶ SetupScreen
  │                                          │
  │                                     (on success)
  │                                          │
  └── ApiClient.isConfigured == true ───▶ MainTabs
                                            ├── Tab 1: DeviceListScreen
                                            │     └──▶ DeviceDetailScreen
                                            │            └──▶ SessionDetailScreen
                                            ├── Tab 2: AllSessionsScreen
                                            │     ├──▶ SessionDetailScreen
                                            │     └──▶ DeviceDetailScreen
                                            └── Tab 3: SettingsScreen
                                                  └── (disconnect) ──▶ SetupScreen

                                        NotificationListScreen (ModalBottomSheet)
                                          └── accessible from bell icon on tabs 1 & 2
```

### Tab Navigation

Three tabs using Material 3 `NavigationBar`:
1. **Devices** — `Icons.Outlined.Devices` (or custom)
2. **Sessions** — `Icons.Outlined.Terminal` (or custom)
3. **Settings** — `Icons.Outlined.Settings`

Each tab maintains its own `NavHost` back stack (standard Compose multi-tab pattern). Swiping between tabs via `HorizontalPager` to match iOS swipe gesture navigation.

---

## 6. Screens (Detailed)

### 6.1 SetupScreen

**Matches iOS:** `SetupView`

**Layout:**
- Centered Claudiator logo (use app icon drawable)
- "Server URL" text field (`KeyboardType.Uri`)
- "API Key" text field (password toggle via `PasswordVisualTransformation`)
- Error message text (red, conditional)
- "Connect" button with loading indicator
- Subtitle/version text

**ViewModel: SetupViewModel**
```kotlin
data class SetupUiState(
    val serverUrl: String = "",
    val apiKey: String = "",
    val isLoading: Boolean = false,
    val error: String? = null,
    val connectionSuccess: Boolean = false
)
```

- `connect()`: validates URL → tests `ping()` → saves credentials → sets `connectionSuccess = true`
- URL validation uses `URLValidator.cleanAndValidate()` (same logic as iOS)
- Error messages match iOS: network unreachable, HTTPS required for remote, timeout, unauthorized, etc.

### 6.2 DeviceListScreen

**Matches iOS:** `DeviceListView`

**Layout:**
- Top bar: "Devices" title + bell icon with unread badge
- Pull-to-refresh via `PullToRefreshBox`
- `LazyColumn` of `DeviceRow` composables
- Empty state: "No Devices"

**DeviceRow composable:**
- Platform icon (Apple/Linux/Windows vector) + device name
- "Last active: X ago" subtitle
- Status badges row: active count (green), waiting count (orange), idle count (yellow)
- Tap → navigate to DeviceDetailScreen

**Auto-refresh:** Triggers `refresh()` when `versionMonitor.dataVersion` changes (collected via `collectAsStateWithLifecycle`).

### 6.3 DeviceDetailScreen

**Matches iOS:** `DeviceDetailView`

**Layout:**
- Top bar with back arrow + device name
- Device header: large platform icon + name + platform label
- Details section: device ID, first seen, last active, active sessions
- Sessions section header with Active/All segmented filter
- `LazyColumn` of `SessionRow` composables
- Pull-to-refresh

**ViewModel: SessionListViewModel** — same as iOS, fetches sessions for a single device with client-side filtering.

### 6.4 AllSessionsScreen

**Matches iOS:** `AllSessionsView`

**Layout:**
- Top bar: "Sessions" title + bell icon with badge + grouping toggle icon
- Active/All segmented filter in toolbar
- Three layout modes (matching iOS):
  1. **Flat list** — `LazyColumn` of `AllSessionRow`
  2. **Grouped (compact)** — `LazyColumn` with `DeviceGroupHeader` (collapsible) + `AllSessionRow` items
  3. **Grouped (expanded/tablet)** — 2-column `LazyVerticalGrid` with `DeviceGroupCard` (collapsible)
- Tablet detection via `WindowSizeClass` — use expanded layout for `WindowWidthSizeClass.Expanded`
- Infinite scroll: sentinel item at bottom triggers `loadMore()`
- Pull-to-refresh
- Notification pulsing animation on rows with unread notifications (matching iOS 1.2s cycle)
- Error overlay

**ViewModel: AllSessionsViewModel** — pagination, grouping, filtering matching iOS exactly.

### 6.5 SessionDetailScreen

**Matches iOS:** `SessionDetailView`

**Layout:**
- Top bar with back arrow + session title (or "Session")
- Status header: colored circle with halo animation + status label + title + cwd
- Collapsible "Details" section (using `AnimatedVisibility`):
  - Device card (tappable → DeviceDetailScreen)
  - Working directory (full + short display)
  - Status + started at + last event timestamps
  - Session ID (copyable)
- "Events" section header
- `LazyColumn` of `EventRow` composables:
  - Event icon (colored by type) + hook event name
  - Tool name badge (if present)
  - Notification type badge (if present)
  - Message text (if present)
  - Relative timestamp
- Pull-to-refresh
- Marks session as read via `AppNotificationManager` on composition

### 6.6 NotificationListScreen

**Matches iOS:** `NotificationListView`

**Presented as:** `ModalBottomSheet` (Material 3)

**Layout:**
- Header: "Notifications" + "Mark All Read" text button
- `LazyColumn` of `NotificationRow` composables:
  - Unread indicator dot (blue)
  - Type icon (permission_prompt / idle_prompt / stop)
  - Title + body + relative timestamp
  - Swipe-to-dismiss marks as read (SwipeToDismiss composable)
- Tap marks as read
- Empty state: "No Notifications"

### 6.7 SettingsScreen

**Matches iOS:** `SettingsView`

**Layout:**
- Claudiator logo + version string
- **Appearance Section:**
  - Mode picker: System / Light / Dark (segmented)
  - Theme selector: horizontal row of `ThemePreviewCard` composables (4 color swatches each)
- **Server Configuration Section:**
  - Editable server URL field
  - Editable API key field (password toggle)
  - Connection status indicator (green dot / red dot + label)
  - "Test Connection" / "Save" button
- **Danger Zone Section:**
  - "Disconnect" button (destructive, red)
  - Confirmation dialog before disconnect
  - Disconnect clears SecureStorage and navigates to SetupScreen

---

## 7. Theming

### Theme Architecture

```kotlin
interface AppTheme {
    val id: String
    val name: String
    val preview: List<Color>  // 4 colors for swatch

    // Status colors
    val statusActive: Color
    val statusWaitingInput: Color
    val statusWaitingPermission: Color
    val statusIdle: Color
    val statusEnded: Color

    // Platform colors
    val platformMac: Color
    val platformLinux: Color
    val platformWindows: Color
    val platformDefault: Color

    // Event colors
    val eventSessionStart: Color
    val eventSessionEnd: Color
    val eventStop: Color
    val eventNotification: Color
    val eventUserPromptSubmit: Color
    val eventDefault: Color

    // Server status
    val serverConnected: Color
    val serverDisconnected: Color

    // UI
    val uiError: Color
    val uiAccent: Color
    val uiTint: Color

    // Surface (light/dark adaptive)
    fun pageBackground(isDark: Boolean): Color
    fun cardBackground(isDark: Boolean): Color
    fun cardBorder(isDark: Boolean): Color
}
```

### Four Themes (matching iOS exactly)

1. **Standard** — iOS system colors adapted to Material: green, blue, orange, red
2. **Neon Ops** — Cyberpunk: #00FF99, #FF4080, #00BFFF, #FFC200
3. **Solarized** — Classic: #859900, #268BD2, #B58900, #DC322F
4. **Arctic** — Cool: #2EB887, #3B82F6, #8B95A5, #EF4444

### ThemeManager

```kotlin
class ThemeManager(context: Context) {
    // Persisted to DataStore Preferences
    val selectedThemeId: StateFlow<String>     // default: "standard"
    val appearanceMode: StateFlow<AppearanceMode>  // System, Light, Dark

    fun selectTheme(themeId: String)
    fun setAppearance(mode: AppearanceMode)
    fun currentTheme(): AppTheme
}

enum class AppearanceMode { SYSTEM, LIGHT, DARK }
```

### Compose Integration

A `CompositionLocal` provides the active theme to all composables:

```kotlin
val LocalAppTheme = staticCompositionLocalOf<AppTheme> { StandardTheme }

@Composable
fun ClaudiatorTheme(
    themeManager: ThemeManager,
    content: @Composable () -> Unit
) {
    val themeId by themeManager.selectedThemeId.collectAsStateWithLifecycle()
    val mode by themeManager.appearanceMode.collectAsStateWithLifecycle()
    val theme = themeManager.currentTheme()

    val darkTheme = when (mode) {
        AppearanceMode.SYSTEM -> isSystemInDarkTheme()
        AppearanceMode.LIGHT -> false
        AppearanceMode.DARK -> true
    }

    MaterialTheme(
        colorScheme = if (darkTheme) darkColorScheme() else lightColorScheme()
    ) {
        CompositionLocalProvider(LocalAppTheme provides theme) {
            content()
        }
    }
}
```

Composables access theme colors via `LocalAppTheme.current.statusActive`, etc.

### Constants (matching iOS)

```kotlin
object ThemeConstants {
    val cardCornerRadius = 12.dp
    const val cardBorderOpacity = 0.3f
    val cardBorderWidth = 0.5.dp
}
```

---

## 8. Push Notifications (FCM)

### Android Side

**AndroidManifest.xml entries:**
```xml
<service
    android:name=".services.FcmService"
    android:exported="false">
    <intent-filter>
        <action android:name="com.google.firebase.MESSAGING_EVENT" />
    </intent-filter>
</service>

<meta-data
    android:name="com.google.firebase.messaging.default_notification_channel_id"
    android:value="claudiator_sessions" />
```

**Notification Channel (created in ClaudiatorApp.onCreate):**
```kotlin
val channel = NotificationChannel(
    "claudiator_sessions",
    "Claudiator Sessions",
    NotificationManager.IMPORTANCE_HIGH
).apply {
    description = "Claude Code session alerts"
    enableVibration(true)
}
notificationManager.createNotificationChannel(channel)
```

**FcmService behavior:**

`onNewToken(token)`:
1. Store token in SharedPreferences (not encrypted — it's not secret)
2. If `ApiClient.isConfigured`, call `registerPushToken(token)` in a coroutine
3. If not configured, token will be registered during setup flow

`onMessageReceived(message)`:
1. Extract fields from `message.data` map: `notification_id`, `session_id`, `device_id`, `title`, `body`, `notification_type`
2. Record `notification_id` in `AppNotificationManager` as push-received
3. Build Android notification:
   - Title and body from data
   - Icon: appropriate type icon (or app icon)
   - Channel: `claudiator_sessions`
   - PendingIntent: deep link to `SessionDetailScreen(sessionId)`
   - Auto-cancel: true
4. Show via `NotificationManagerCompat.notify()`

**Deduplication (matching iOS):**
- `AppNotificationManager` maintains a `pushReceivedIds: MutableSet<String>` with timestamps
- IDs older than 10 minutes are pruned on each insert
- When polling fetches notifications, any ID already in `pushReceivedIds` skips local notification display
- Prevents duplicate alerts from both FCM push and polling paths

### Server Side (FCM Integration)

**New server dependency:** HTTP client for FCM v1 API (the server already uses `reqwest`, reuse it).

**Configuration:**
- New env var: `CLAUDIATOR_FCM_SERVICE_ACCOUNT` — path to Firebase service account JSON file
- Server reads this on startup, caches the OAuth2 access token (auto-refreshes before expiry)

**FCM Send Logic:**

When the server creates a notification and needs to push:
1. Query push tokens where `platform = 'android'` for the authenticated API key
2. For each Android token, send via FCM HTTP v1 API:
   ```
   POST https://fcm.googleapis.com/v1/projects/{project_id}/messages:send
   Authorization: Bearer {oauth2_token}
   Content-Type: application/json

   {
     "message": {
       "token": "{device_fcm_token}",
       "data": {
         "notification_id": "...",
         "session_id": "...",
         "device_id": "...",
         "title": "...",
         "body": "...",
         "notification_type": "..."
       }
     }
   }
   ```
3. Uses **data-only messages** (no `notification` block) so the app's `onMessageReceived` always fires, even in foreground — giving us full control over display and deduplication.

**OAuth2 Token Management:**
- Parse service account JSON for `client_email`, `private_key`, `token_uri`
- Generate JWT, exchange for access token
- Cache token, refresh when within 5 minutes of expiry
- Use `jsonwebtoken` crate (already commonly used in Rust) for JWT signing

**Existing APNs path unchanged.** The push dispatch becomes:
```rust
match token.platform.as_str() {
    "ios" => send_apns(token, payload).await,
    "android" => send_fcm(token, payload).await,
    _ => warn!("Unknown push platform: {}", token.platform),
}
```

**New documentation:** `server/FCM_SETUP.md` explaining:
1. Create Firebase project
2. Add Android app to Firebase project
3. Download `google-services.json` → place in `android/app/`
4. Generate service account key → place on server
5. Set `CLAUDIATOR_FCM_SERVICE_ACCOUNT=/path/to/key.json`

---

## 9. Shared Components

### ThemedCard

```kotlin
@Composable
fun ThemedCard(
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit
)
```
Applies theme's card background, border (with opacity), corner radius, and subtle shadow. Direct equivalent of iOS `.themedCard()` modifier.

### ThemedPage

Applies theme's page background as the screen's root surface. Equivalent of iOS `.themedPage()`.

### PlatformIcon

Renders platform vector icon (Apple, Linux, Windows penguin, default monitor) with theme-appropriate color. Takes `platform: String` and `size: Dp`.

### StatusBadge

Colored pill with count + label. E.g., green pill "2 active". Takes `count: Int`, `label: String`, `color: Color`.

### ThemedSegmentedPicker

Material 3 segmented button row with theme accent colors. Used for Active/All filters.

### ThemePreviewCard

Small card showing 4 color swatches + theme name. Tappable for selection. Highlighted border when selected.

---

## 10. Utilities

### URLValidator

Port of iOS `URLValidator.cleanAndValidate()`:
- Trims whitespace
- Detects local URLs (localhost, 127.0.0.1, .local) → prepends `http://`
- Remote URLs → prepends `https://`
- Validates via `URL` / `URI` parsing
- Returns cleaned URL string or null

### TimeFormatting

Port of iOS `relativeTime(isoString)`:
- Parses ISO 8601 timestamps
- Returns relative strings: "just now", "2 minutes ago", "3 hours ago", "yesterday", "Mar 15"

### StringExtensions

- `cwdShortDisplay(path)` — extracts last 2 path components
- `statusDisplayLabel(status)` — "waiting_for_input" → "Waiting for Input"
- `priorityStatus(sessions)` — highest priority status from collection

---

## 11. Testing

Mirror the iOS test suite. All tests are JVM unit tests (no Android instrumentation needed for these).

### Test Files

| Test File | Tests |
|---|---|
| `ModelDecodingTests.kt` | JSON deserialization for all models, edge cases (null fields, unknown fields) |
| `URLValidatorTests.kt` | URL cleaning, local vs remote detection, edge cases |
| `HelperTests.kt` | `relativeTime`, `cwdShortDisplay`, `statusDisplayLabel`, `priorityStatus` |
| `SetupViewModelTests.kt` | Connect flow, validation errors, success path |
| `ViewModelTests.kt` | DeviceListViewModel, SessionListViewModel, AllSessionsViewModel, EventListViewModel refresh/filter/pagination |
| `NotificationManagerTests.kt` | Fetch, dedup, mark read, caps, push-received tracking |
| `ThemeTests.kt` | Theme selection persistence, all themes have required colors |

### Test Approach

- Use `kotlinx-coroutines-test` for coroutine testing (`runTest`, `TestDispatcher`)
- Fake `ApiClient` implementation for ViewModel tests (interface extraction or open class)
- No mocking library needed — simple fakes matching the service interfaces
- Test state emissions via `StateFlow` collection in test scope

---

## 12. Android Manifest

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android">

    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />

    <application
        android:name=".ClaudiatorApp"
        android:allowBackup="true"
        android:icon="@mipmap/ic_launcher"
        android:label="Claudiator"
        android:supportsRtl="true"
        android:theme="@style/Theme.Claudiator"
        android:usesCleartextTraffic="true">

        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:theme="@style/Theme.Claudiator">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>

        <service
            android:name=".services.FcmService"
            android:exported="false">
            <intent-filter>
                <action android:name="com.google.firebase.MESSAGING_EVENT" />
            </intent-filter>
        </service>

        <meta-data
            android:name="com.google.firebase.messaging.default_notification_channel_id"
            android:value="claudiator_sessions" />

    </application>
</manifest>
```

`usesCleartextTraffic="true"` is needed for local/development servers using HTTP (matching iOS's local networking permission). For production, consider a network security config that restricts cleartext to localhost only.

---

## 13. README.md (android/)

Document:
- Prerequisites (Android Studio, JDK 17+)
- Firebase setup (link to FCM_SETUP.md)
- Building (`./gradlew assembleDebug`)
- Running tests (`./gradlew test`)
- Signing for release
- Play Store submission notes

---

## 14. Out of Scope

These items are explicitly NOT part of this spec:
- Local database / offline caching (iOS doesn't have it either)
- Widget support
- Wear OS companion
- Deep link handling from URLs (only from notification taps)
- CI/CD workflows for Android (can be added separately)
- Automated UI tests (only unit tests, matching iOS)

---

## 15. Summary of Server Changes

The server changes are limited and surgical:

1. **New module:** `fcm.rs` — FCM HTTP v1 API client with OAuth2 token management
2. **Modified:** Push dispatch logic to branch on `platform` field (ios → APNs, android → FCM)
3. **New config:** `CLAUDIATOR_FCM_SERVICE_ACCOUNT` env var
4. **New doc:** `server/FCM_SETUP.md`
5. **No API changes** — all endpoints remain identical; the push register endpoint already accepts `platform: "android"`
