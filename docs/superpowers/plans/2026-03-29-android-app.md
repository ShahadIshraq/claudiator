# Claudiator Android App Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Kotlin + Jetpack Compose Android app that is a 1:1 feature port of the existing Claudiator iOS app.

**Architecture:** Single-Activity MVVM with Compose Navigation. ViewModels expose state via StateFlow, Compose screens collect via collectAsStateWithLifecycle. Services instantiated in Application class and passed through.

**Tech Stack:** Kotlin 2.0, Jetpack Compose (Material 3), Ktor Client, kotlinx.serialization, Firebase Cloud Messaging, EncryptedSharedPreferences, DataStore Preferences.

**Spec:** `docs/superpowers/specs/2026-03-29-android-app-design.md`

---

## File Map

```
android/
├── build.gradle.kts                          # Root build script
├── settings.gradle.kts                       # Module settings
├── gradle.properties                         # Build properties
├── .gitignore                                # Android ignores
├── README.md                                 # Build/setup docs
├── app/
│   ├── build.gradle.kts                      # App module build
│   ├── proguard-rules.pro                    # Release minification
│   └── src/
│       ├── main/
│       │   ├── AndroidManifest.xml
│       │   ├── res/
│       │   │   ├── values/strings.xml
│       │   │   ├── values/themes.xml
│       │   │   ├── drawable/ic_launcher_foreground.xml
│       │   │   └── mipmap-*/ic_launcher.webp
│       │   └── java/com/claudiator/app/
│       │       ├── ClaudiatorApp.kt          # Application class, DI
│       │       ├── MainActivity.kt           # Single activity host
│       │       ├── navigation/
│       │       │   ├── Screen.kt             # Route definitions
│       │       │   └── AppNavigation.kt      # NavHost + tab scaffold
│       │       ├── models/
│       │       │   ├── Device.kt
│       │       │   ├── Session.kt
│       │       │   ├── Event.kt
│       │       │   ├── AppNotification.kt
│       │       │   └── ApiResponses.kt       # Wrappers + SessionListPage
│       │       ├── services/
│       │       │   ├── ApiClient.kt
│       │       │   ├── SecureStorage.kt
│       │       │   ├── AppNotificationManager.kt
│       │       │   ├── VersionMonitor.kt
│       │       │   └── FcmService.kt
│       │       ├── viewmodels/
│       │       │   ├── SetupViewModel.kt
│       │       │   ├── DeviceListViewModel.kt
│       │       │   ├── SessionListViewModel.kt
│       │       │   ├── AllSessionsViewModel.kt
│       │       │   └── EventListViewModel.kt
│       │       ├── ui/
│       │       │   ├── theme/
│       │       │   │   ├── AppTheme.kt       # Theme interface + color helpers
│       │       │   │   ├── Themes.kt         # 4 built-in themes
│       │       │   │   ├── ThemeManager.kt   # Persistence + state
│       │       │   │   └── ClaudiatorTheme.kt # CompositionLocal provider
│       │       │   ├── setup/
│       │       │   │   └── SetupScreen.kt
│       │       │   ├── devices/
│       │       │   │   ├── DeviceListScreen.kt
│       │       │   │   ├── DeviceDetailScreen.kt
│       │       │   │   └── DeviceRow.kt
│       │       │   ├── sessions/
│       │       │   │   ├── AllSessionsScreen.kt
│       │       │   │   ├── SessionDetailScreen.kt
│       │       │   │   ├── SessionRow.kt
│       │       │   │   ├── AllSessionRow.kt
│       │       │   │   ├── DeviceGroupHeader.kt
│       │       │   │   ├── DeviceGroupCard.kt
│       │       │   │   └── EventRow.kt
│       │       │   ├── notifications/
│       │       │   │   ├── NotificationListSheet.kt
│       │       │   │   └── NotificationRow.kt
│       │       │   ├── settings/
│       │       │   │   ├── SettingsScreen.kt
│       │       │   │   ├── AppearanceSection.kt
│       │       │   │   ├── ServerConfigSection.kt
│       │       │   │   └── DangerZoneSection.kt
│       │       │   └── components/
│       │       │       ├── ThemedCard.kt
│       │       │       ├── PlatformIcon.kt
│       │       │       ├── StatusBadge.kt
│       │       │       ├── ThemedSegmentedPicker.kt
│       │       │       └── ThemePreviewCard.kt
│       │       └── util/
│       │           ├── URLValidator.kt
│       │           └── Helpers.kt            # relativeTime, cwdShortDisplay, etc.
│       └── test/java/com/claudiator/app/
│           ├── URLValidatorTest.kt
│           ├── HelpersTest.kt
│           ├── ModelDecodingTest.kt
│           ├── ThemeTest.kt
│           ├── ViewModelTest.kt
│           ├── SetupViewModelTest.kt
│           └── NotificationManagerTest.kt
```

---

### Task 1: Project Scaffolding & Build Configuration

**Files:**
- Create: `android/build.gradle.kts`
- Create: `android/settings.gradle.kts`
- Create: `android/gradle.properties`
- Create: `android/.gitignore`
- Create: `android/app/build.gradle.kts`
- Create: `android/app/proguard-rules.pro`
- Create: `android/app/src/main/AndroidManifest.xml`
- Create: `android/app/src/main/res/values/strings.xml`
- Create: `android/app/src/main/res/values/themes.xml`

- [ ] **Step 1: Create root build.gradle.kts**

```kotlin
// android/build.gradle.kts
plugins {
    id("com.android.application") version "8.7.3" apply false
    id("org.jetbrains.kotlin.android") version "2.1.0" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.1.0" apply false
    id("org.jetbrains.kotlin.plugin.serialization") version "2.1.0" apply false
    id("com.google.gms.google-services") version "4.4.2" apply false
}
```

- [ ] **Step 2: Create settings.gradle.kts**

```kotlin
// android/settings.gradle.kts
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolution {
    @Suppress("UnstableApiUsage")
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "Claudiator"
include(":app")
```

- [ ] **Step 3: Create gradle.properties**

```properties
# android/gradle.properties
org.gradle.jvmargs=-Xmx2048m -Dfile.encoding=UTF-8
android.useAndroidX=true
kotlin.code.style=official
android.nonTransitiveRClass=true
```

- [ ] **Step 4: Create .gitignore**

```
# android/.gitignore
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
/captures
.externalNativeBuild/
.cxx/
```

- [ ] **Step 5: Create app/build.gradle.kts**

```kotlin
// android/app/build.gradle.kts
plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
    id("org.jetbrains.kotlin.plugin.serialization")
    id("com.google.gms.google-services")
}

android {
    namespace = "com.claudiator.app"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.claudiator.app"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "1.0.0"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        compose = true
    }
}

dependencies {
    // Compose BOM
    val composeBom = platform("androidx.compose:compose-bom:2024.12.01")
    implementation(composeBom)
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3-window-size-class")
    debugImplementation("androidx.compose.ui:ui-tooling")

    // Core
    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.7")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.8.7")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.8.7")
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.navigation:navigation-compose:2.8.5")

    // Networking
    implementation("io.ktor:ktor-client-android:3.0.3")
    implementation("io.ktor:ktor-client-content-negotiation:3.0.3")
    implementation("io.ktor:ktor-serialization-kotlinx-json:3.0.3")

    // JSON
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")

    // Secure Storage
    implementation("androidx.security:security-crypto:1.1.0-alpha06")

    // DataStore
    implementation("androidx.datastore:datastore-preferences:1.1.1")

    // Firebase
    implementation(platform("com.google.firebase:firebase-bom:33.7.0"))
    implementation("com.google.firebase:firebase-messaging-ktx")

    // Testing
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.9.0")
    testImplementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")
}
```

- [ ] **Step 6: Create proguard-rules.pro**

```pro
# android/app/proguard-rules.pro
# Keep kotlinx.serialization
-keepattributes *Annotation*, InnerClasses
-dontnote kotlinx.serialization.AnnotationsKt

-keepclassmembers class kotlinx.serialization.json.** {
    *** Companion;
}
-keepclasseswithmembers class kotlinx.serialization.json.** {
    kotlinx.serialization.KSerializer serializer(...);
}

-keep,includedescriptorclasses class com.claudiator.app.models.**$$serializer { *; }
-keepclassmembers class com.claudiator.app.models.** {
    *** Companion;
}
-keepclasseswithmembers class com.claudiator.app.models.** {
    kotlinx.serialization.KSerializer serializer(...);
}

# Keep Ktor
-keep class io.ktor.** { *; }
-dontwarn io.ktor.**
```

- [ ] **Step 7: Create AndroidManifest.xml**

```xml
<?xml version="1.0" encoding="utf-8"?>
<!-- android/app/src/main/AndroidManifest.xml -->
<manifest xmlns:android="http://schemas.android.com/apk/res/android">

    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />

    <application
        android:name=".ClaudiatorApp"
        android:allowBackup="true"
        android:icon="@mipmap/ic_launcher"
        android:label="@string/app_name"
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

- [ ] **Step 8: Create res/values/strings.xml**

```xml
<!-- android/app/src/main/res/values/strings.xml -->
<resources>
    <string name="app_name">Claudiator</string>
    <string name="tab_devices">Devices</string>
    <string name="tab_sessions">Sessions</string>
    <string name="tab_settings">Settings</string>
    <string name="no_devices">No Devices</string>
    <string name="no_sessions">No Sessions</string>
    <string name="no_notifications">No Notifications</string>
    <string name="connect">Connect</string>
    <string name="disconnect">Disconnect</string>
    <string name="save">Save</string>
    <string name="server_url">Server URL</string>
    <string name="api_key">API Key</string>
    <string name="mark_all_read">Mark All Read</string>
    <string name="active">Active</string>
    <string name="all">All</string>
    <string name="notifications">Notifications</string>
    <string name="notification_channel_name">Claudiator Sessions</string>
    <string name="notification_channel_description">Claude Code session alerts</string>
    <string name="disconnect_confirm_title">Disconnect?</string>
    <string name="disconnect_confirm_message">This will remove your server configuration and API key.</string>
    <string name="cancel">Cancel</string>
</resources>
```

- [ ] **Step 9: Create res/values/themes.xml**

```xml
<!-- android/app/src/main/res/values/themes.xml -->
<resources>
    <style name="Theme.Claudiator" parent="android:Theme.Material.Light.NoActionBar">
        <item name="android:statusBarColor">@android:color/transparent</item>
        <item name="android:navigationBarColor">@android:color/transparent</item>
    </style>
</resources>
```

- [ ] **Step 10: Set up Gradle wrapper**

Run from the `android/` directory:
```bash
cd android && gradle wrapper --gradle-version 8.11.1
```

This generates `gradlew`, `gradlew.bat`, and `gradle/wrapper/` files.

- [ ] **Step 11: Verify build compiles**

```bash
cd android && ./gradlew assembleDebug
```

Expected: BUILD SUCCESSFUL (will fail until we add the source files, but the build configuration should parse correctly). If the google-services plugin fails because `google-services.json` is missing, temporarily comment out the plugin line, verify the rest compiles, then uncomment.

- [ ] **Step 12: Commit**

```bash
git add android/
git commit -m "scaffold android project"
```

---

### Task 2: Data Models & JSON Serialization

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/models/Device.kt`
- Create: `android/app/src/main/java/com/claudiator/app/models/Session.kt`
- Create: `android/app/src/main/java/com/claudiator/app/models/Event.kt`
- Create: `android/app/src/main/java/com/claudiator/app/models/AppNotification.kt`
- Create: `android/app/src/main/java/com/claudiator/app/models/ApiResponses.kt`
- Create: `android/app/src/test/java/com/claudiator/app/ModelDecodingTest.kt`

- [ ] **Step 1: Write ModelDecodingTest.kt**

```kotlin
// android/app/src/test/java/com/claudiator/app/ModelDecodingTest.kt
package com.claudiator.app

import com.claudiator.app.models.*
import kotlinx.serialization.json.Json
import org.junit.Assert.*
import org.junit.Test

class ModelDecodingTest {

    private val json = Json { ignoreUnknownKeys = true }

    // -- Device --

    @Test
    fun `decode Device with all fields`() {
        val raw = """
        {
            "device_id": "dev_789",
            "device_name": "Linux Server",
            "platform": "linux",
            "first_seen": "2024-01-10T08:00:00Z",
            "last_seen": "2024-01-15T12:00:00Z",
            "active_sessions": 3
        }
        """
        val device = json.decodeFromString<Device>(raw)
        assertEquals("dev_789", device.deviceId)
        assertEquals("Linux Server", device.deviceName)
        assertEquals("linux", device.platform)
        assertEquals("2024-01-10T08:00:00Z", device.firstSeen)
        assertEquals("2024-01-15T12:00:00Z", device.lastSeen)
        assertEquals(3, device.activeSessions)
    }

    // -- Session --

    @Test
    fun `decode Session with all fields`() {
        val raw = """
        {
            "session_id": "sess_123",
            "device_id": "dev_456",
            "started_at": "2024-01-15T10:30:00Z",
            "last_event": "2024-01-15T11:00:00Z",
            "status": "active",
            "cwd": "/Users/test/project",
            "title": "Test Session",
            "device_name": "MacBook Pro",
            "platform": "darwin"
        }
        """
        val session = json.decodeFromString<Session>(raw)
        assertEquals("sess_123", session.sessionId)
        assertEquals("dev_456", session.deviceId)
        assertEquals("active", session.status)
        assertEquals("/Users/test/project", session.cwd)
        assertEquals("Test Session", session.title)
        assertEquals("MacBook Pro", session.deviceName)
        assertEquals("darwin", session.platform)
    }

    @Test
    fun `decode Session with missing optional fields`() {
        val raw = """
        {
            "session_id": "sess_123",
            "device_id": "dev_456",
            "started_at": "2024-01-15T10:30:00Z",
            "last_event": "2024-01-15T11:00:00Z",
            "status": "idle"
        }
        """
        val session = json.decodeFromString<Session>(raw)
        assertEquals("sess_123", session.sessionId)
        assertNull(session.cwd)
        assertNull(session.title)
        assertNull(session.deviceName)
        assertNull(session.platform)
    }

    @Test
    fun `decode Session with unknown status`() {
        val raw = """
        {
            "session_id": "sess_123",
            "device_id": "dev_456",
            "started_at": "2024-01-15T10:30:00Z",
            "last_event": "2024-01-15T11:00:00Z",
            "status": "unknown_status"
        }
        """
        val session = json.decodeFromString<Session>(raw)
        assertEquals("unknown_status", session.status)
    }

    // -- Event --

    @Test
    fun `decode Event with all fields`() {
        val raw = """
        {
            "id": 42,
            "hook_event_name": "SessionStart",
            "timestamp": "2024-01-15T10:30:00Z",
            "tool_name": "Read",
            "notification_type": "info",
            "message": "Session started"
        }
        """
        val event = json.decodeFromString<Event>(raw)
        assertEquals(42, event.id)
        assertEquals("SessionStart", event.hookEventName)
        assertEquals("Read", event.toolName)
        assertEquals("info", event.notificationType)
        assertEquals("Session started", event.message)
    }

    @Test
    fun `decode Event with missing optional fields`() {
        val raw = """
        {
            "id": 99,
            "hook_event_name": "Stop",
            "timestamp": "2024-01-15T12:00:00Z"
        }
        """
        val event = json.decodeFromString<Event>(raw)
        assertEquals(99, event.id)
        assertEquals("Stop", event.hookEventName)
        assertNull(event.toolName)
        assertNull(event.notificationType)
        assertNull(event.message)
    }

    // -- AppNotification --

    @Test
    fun `decode AppNotification with all fields`() {
        val raw = """
        {
            "id": "notif_123",
            "session_id": "sess_456",
            "device_id": "dev_789",
            "title": "New Event",
            "body": "Tool execution completed",
            "notification_type": "info",
            "payload_json": "{\"key\": \"value\"}",
            "created_at": "2024-01-15T14:00:00Z",
            "acknowledged": false
        }
        """
        val notif = json.decodeFromString<AppNotification>(raw)
        assertEquals("notif_123", notif.notificationId)
        assertEquals("sess_456", notif.sessionId)
        assertEquals("dev_789", notif.deviceId)
        assertEquals("New Event", notif.title)
        assertEquals("Tool execution completed", notif.body)
        assertEquals("info", notif.notificationType)
        assertEquals("""{"key": "value"}""", notif.payloadJson)
        assertEquals("2024-01-15T14:00:00Z", notif.createdAt)
        assertEquals(false, notif.acknowledged)
    }

    @Test
    fun `decode AppNotification with missing optional fields`() {
        val raw = """
        {
            "id": "notif_456",
            "session_id": "sess_789",
            "device_id": "dev_123",
            "title": "Test",
            "body": "Test body",
            "notification_type": "warning",
            "created_at": "2024-01-15T15:00:00Z"
        }
        """
        val notif = json.decodeFromString<AppNotification>(raw)
        assertEquals("notif_456", notif.notificationId)
        assertNull(notif.payloadJson)
        assertNull(notif.acknowledged)
    }

    // -- API Responses --

    @Test
    fun `decode PingResponse`() {
        val raw = """
        {
            "status": "ok",
            "server_version": "0.4.3",
            "data_version": 42,
            "notification_version": 7
        }
        """
        val ping = json.decodeFromString<PingResponse>(raw)
        assertEquals("ok", ping.status)
        assertEquals(42L, ping.dataVersion)
        assertEquals(7L, ping.notificationVersion)
    }

    @Test
    fun `decode SessionListPage`() {
        val raw = """
        {
            "sessions": [
                {
                    "session_id": "s1",
                    "device_id": "d1",
                    "started_at": "2024-01-15T10:00:00Z",
                    "last_event": "2024-01-15T11:00:00Z",
                    "status": "active"
                }
            ],
            "has_more": true,
            "next_offset": 50
        }
        """
        val page = json.decodeFromString<SessionListPage>(raw)
        assertEquals(1, page.sessions.size)
        assertTrue(page.hasMore)
        assertEquals(50, page.nextOffset)
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd android && ./gradlew test
```

Expected: compilation error — model classes don't exist yet.

- [ ] **Step 3: Create Device.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/models/Device.kt
package com.claudiator.app.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class Device(
    @SerialName("device_id") val deviceId: String,
    @SerialName("device_name") val deviceName: String,
    val platform: String,
    @SerialName("first_seen") val firstSeen: String,
    @SerialName("last_seen") val lastSeen: String,
    @SerialName("active_sessions") val activeSessions: Int,
)
```

- [ ] **Step 4: Create Session.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/models/Session.kt
package com.claudiator.app.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

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
    val platform: String? = null,
)
```

- [ ] **Step 5: Create Event.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/models/Event.kt
package com.claudiator.app.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class Event(
    val id: Int,
    @SerialName("hook_event_name") val hookEventName: String,
    val timestamp: String,
    @SerialName("tool_name") val toolName: String? = null,
    @SerialName("notification_type") val notificationType: String? = null,
    val message: String? = null,
)
```

- [ ] **Step 6: Create AppNotification.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/models/AppNotification.kt
package com.claudiator.app.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

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
    val acknowledged: Boolean? = null,
)
```

- [ ] **Step 7: Create ApiResponses.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/models/ApiResponses.kt
package com.claudiator.app.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class PingResponse(
    val status: String,
    @SerialName("server_version") val serverVersion: String? = null,
    @SerialName("data_version") val dataVersion: Long = 0,
    @SerialName("notification_version") val notificationVersion: Long = 0,
)

@Serializable
data class DevicesResponse(val devices: List<Device>)

@Serializable
data class SessionsResponse(val sessions: List<Session>)

@Serializable
data class SessionListPage(
    val sessions: List<Session>,
    @SerialName("has_more") val hasMore: Boolean,
    @SerialName("next_offset") val nextOffset: Int,
)

@Serializable
data class EventsResponse(val events: List<Event>)

@Serializable
data class NotificationsResponse(val notifications: List<AppNotification>)
```

- [ ] **Step 8: Run tests to verify they pass**

```bash
cd android && ./gradlew test
```

Expected: all ModelDecodingTest tests PASS.

- [ ] **Step 9: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/models/ android/app/src/test/java/com/claudiator/app/ModelDecodingTest.kt
git commit -m "add android data models with tests"
```

---

### Task 3: Utility Functions

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/util/URLValidator.kt`
- Create: `android/app/src/main/java/com/claudiator/app/util/Helpers.kt`
- Create: `android/app/src/test/java/com/claudiator/app/URLValidatorTest.kt`
- Create: `android/app/src/test/java/com/claudiator/app/HelpersTest.kt`

- [ ] **Step 1: Write URLValidatorTest.kt**

```kotlin
// android/app/src/test/java/com/claudiator/app/URLValidatorTest.kt
package com.claudiator.app

import com.claudiator.app.util.URLValidator
import org.junit.Assert.*
import org.junit.Test

class URLValidatorTest {

    @Test
    fun `empty input returns null`() {
        assertNull(URLValidator.cleanAndValidate(""))
    }

    @Test
    fun `whitespace-only input returns null`() {
        assertNull(URLValidator.cleanAndValidate("   "))
        assertNull(URLValidator.cleanAndValidate("\t\n"))
    }

    @Test
    fun `trailing slash is removed`() {
        assertEquals("https://example.com", URLValidator.cleanAndValidate("https://example.com/"))
    }

    @Test
    fun `no trailing slash is left unchanged`() {
        assertEquals("https://example.com", URLValidator.cleanAndValidate("https://example.com"))
    }

    @Test
    fun `localhost gets http prefix`() {
        assertEquals("http://localhost:3000", URLValidator.cleanAndValidate("localhost:3000"))
    }

    @Test
    fun `local domain gets http prefix`() {
        assertEquals("http://server.local", URLValidator.cleanAndValidate("server.local"))
        assertEquals("http://test.local:3000", URLValidator.cleanAndValidate("test.local:3000"))
    }

    @Test
    fun `loopback gets http prefix`() {
        assertEquals("http://127.0.0.1:8080", URLValidator.cleanAndValidate("127.0.0.1:8080"))
    }

    @Test
    fun `remote hostname gets https prefix`() {
        assertEquals("https://example.com", URLValidator.cleanAndValidate("example.com"))
        assertEquals("https://api.myapp.io", URLValidator.cleanAndValidate("api.myapp.io"))
    }

    @Test
    fun `existing http prefix is preserved`() {
        assertEquals("http://example.com", URLValidator.cleanAndValidate("http://example.com"))
    }

    @Test
    fun `existing https prefix is preserved`() {
        assertEquals("https://example.com", URLValidator.cleanAndValidate("https://example.com"))
    }

    @Test
    fun `whitespace trimming and trailing slash combined`() {
        assertEquals("https://example.com", URLValidator.cleanAndValidate("  https://example.com/  "))
    }

    @Test
    fun `localhost with trailing slash gets cleaned`() {
        assertEquals("http://localhost:3000", URLValidator.cleanAndValidate("localhost:3000/"))
    }

    @Test
    fun `invalid URL returns null`() {
        assertNull(URLValidator.cleanAndValidate("http://exam ple.com"))
    }
}
```

- [ ] **Step 2: Write HelpersTest.kt**

```kotlin
// android/app/src/test/java/com/claudiator/app/HelpersTest.kt
package com.claudiator.app

import com.claudiator.app.models.Session
import com.claudiator.app.util.*
import org.junit.Assert.*
import org.junit.Test

class HelpersTest {

    // -- cwdShortDisplay --

    @Test
    fun `cwdShortDisplay with Unix path`() {
        assertEquals("workspace/project", cwdShortDisplay("/Users/test/workspace/project"))
    }

    @Test
    fun `cwdShortDisplay with single component`() {
        assertEquals("/project", cwdShortDisplay("/project"))
    }

    @Test
    fun `cwdShortDisplay with Windows path`() {
        assertEquals("Documents/project", cwdShortDisplay("C:\\Users\\test\\Documents\\project"))
    }

    @Test
    fun `cwdShortDisplay with exactly two components`() {
        assertEquals("workspace/project", cwdShortDisplay("/workspace/project"))
    }

    // -- statusDisplayLabel --

    @Test
    fun `statusDisplayLabel for known statuses`() {
        assertEquals("Active", statusDisplayLabel("active"))
        assertEquals("Waiting for Input", statusDisplayLabel("waiting_for_input"))
        assertEquals("Waiting for Permission", statusDisplayLabel("waiting_for_permission"))
        assertEquals("Idle", statusDisplayLabel("idle"))
        assertEquals("Ended", statusDisplayLabel("ended"))
    }

    @Test
    fun `statusDisplayLabel for unknown status`() {
        assertEquals("Custom Status", statusDisplayLabel("custom_status"))
    }

    // -- priorityStatus --

    private fun session(id: String, status: String) = Session(
        sessionId = id,
        deviceId = "dev1",
        startedAt = "2024-01-15T10:00:00Z",
        lastEvent = "2024-01-15T11:00:00Z",
        status = status,
    )

    @Test
    fun `priorityStatus returns waiting_for_permission when present`() {
        val sessions = listOf(
            session("1", "idle"),
            session("2", "waiting_for_permission"),
            session("3", "active"),
        )
        assertEquals("waiting_for_permission", priorityStatus(sessions))
    }

    @Test
    fun `priorityStatus returns waiting_for_input when no permission wait`() {
        val sessions = listOf(session("1", "idle"), session("2", "waiting_for_input"))
        assertEquals("waiting_for_input", priorityStatus(sessions))
    }

    @Test
    fun `priorityStatus returns active when no waiting states`() {
        val sessions = listOf(session("1", "active"), session("2", "idle"))
        assertEquals("active", priorityStatus(sessions))
    }

    @Test
    fun `priorityStatus returns idle when only idle and ended`() {
        val sessions = listOf(session("1", "idle"), session("2", "ended"))
        assertEquals("idle", priorityStatus(sessions))
    }

    @Test
    fun `priorityStatus returns ended for all ended sessions`() {
        val sessions = listOf(session("1", "ended"))
        assertEquals("ended", priorityStatus(sessions))
    }

    @Test
    fun `priorityStatus returns ended for empty list`() {
        assertEquals("ended", priorityStatus(emptyList()))
    }

    // -- platformIconName --

    @Test
    fun `platformIconName returns correct name`() {
        assertEquals("apple", platformIconName("mac"))
        assertEquals("apple", platformIconName("macos"))
        assertEquals("apple", platformIconName("darwin"))
        assertEquals("linux", platformIconName("linux"))
        assertEquals("windows", platformIconName("windows"))
        assertEquals("monitor", platformIconName("unknown"))
    }

    @Test
    fun `platformIconName is case insensitive`() {
        assertEquals("apple", platformIconName("MAC"))
        assertEquals("linux", platformIconName("Linux"))
        assertEquals("windows", platformIconName("WINDOWS"))
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cd android && ./gradlew test
```

Expected: compilation error.

- [ ] **Step 4: Create URLValidator.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/util/URLValidator.kt
package com.claudiator.app.util

import java.net.URI

object URLValidator {
    fun cleanAndValidate(input: String): String? {
        var url = input.trim()
        if (url.isEmpty()) return null
        if (url.endsWith("/")) url = url.dropLast(1)
        if (!url.startsWith("http")) {
            val isLocal = url.contains(".local") ||
                url.startsWith("localhost") ||
                url.startsWith("127.0.0.1")
            url = (if (isLocal) "http://" else "https://") + url
        }
        return try {
            URI(url).toURL() // validates the URL
            url
        } catch (_: Exception) {
            null
        }
    }
}
```

- [ ] **Step 5: Create Helpers.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/util/Helpers.kt
package com.claudiator.app.util

import com.claudiator.app.models.Session
import java.time.Duration
import java.time.Instant
import java.time.format.DateTimeParseException

fun relativeTime(isoString: String): String {
    val instant = try {
        Instant.parse(isoString)
    } catch (_: DateTimeParseException) {
        return isoString
    }
    val now = Instant.now()
    val duration = Duration.between(instant, now)
    val seconds = duration.seconds
    return when {
        seconds < 0 -> "just now"
        seconds < 60 -> "${seconds}s ago"
        seconds < 3600 -> "${seconds / 60}m ago"
        seconds < 86400 -> "${seconds / 3600}h ago"
        seconds < 604800 -> "${seconds / 86400}d ago"
        else -> {
            val days = seconds / 86400
            "${days / 7}w ago"
        }
    }
}

fun cwdShortDisplay(cwd: String): String {
    val components = cwd.split("/")
    if (components.size >= 2) {
        val nonEmpty = components.filter { it.isNotEmpty() }
        if (nonEmpty.size >= 2) {
            return nonEmpty.takeLast(2).joinToString("/")
        }
    }
    val winComponents = cwd.split("\\")
    if (winComponents.size >= 2) {
        val nonEmpty = winComponents.filter { it.isNotEmpty() }
        if (nonEmpty.size >= 2) {
            return nonEmpty.takeLast(2).joinToString("/")
        }
    }
    return cwd
}

fun statusDisplayLabel(status: String): String = when (status) {
    "active" -> "Active"
    "waiting_for_input" -> "Waiting for Input"
    "waiting_for_permission" -> "Waiting for Permission"
    "idle" -> "Idle"
    "ended" -> "Ended"
    else -> status.replace("_", " ").split(" ").joinToString(" ") { word ->
        word.replaceFirstChar { it.uppercaseChar() }
    }
}

fun priorityStatus(sessions: List<Session>): String {
    if (sessions.any { it.status == "waiting_for_permission" }) return "waiting_for_permission"
    if (sessions.any { it.status == "waiting_for_input" }) return "waiting_for_input"
    if (sessions.any { it.status == "active" }) return "active"
    if (sessions.any { it.status == "idle" }) return "idle"
    return "ended"
}

fun platformIconName(platform: String): String = when (platform.lowercase()) {
    "mac", "macos", "darwin" -> "apple"
    "linux" -> "linux"
    "windows" -> "windows"
    else -> "monitor"
}
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
cd android && ./gradlew test
```

Expected: all URLValidatorTest and HelpersTest tests PASS.

- [ ] **Step 7: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/util/ android/app/src/test/java/com/claudiator/app/URLValidatorTest.kt android/app/src/test/java/com/claudiator/app/HelpersTest.kt
git commit -m "add utility functions with tests"
```

---

### Task 4: Theme System

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/ui/theme/AppTheme.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/theme/Themes.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/theme/ThemeManager.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/theme/ClaudiatorTheme.kt`
- Create: `android/app/src/test/java/com/claudiator/app/ThemeTest.kt`

- [ ] **Step 1: Write ThemeTest.kt**

```kotlin
// android/app/src/test/java/com/claudiator/app/ThemeTest.kt
package com.claudiator.app

import androidx.compose.ui.graphics.Color
import com.claudiator.app.ui.theme.*
import org.junit.Assert.*
import org.junit.Test

class ThemeTest {

    private val theme = StandardTheme

    @Test
    fun `statusColor returns correct color for known statuses`() {
        assertEquals(theme.statusActive, theme.statusColor("active"))
        assertEquals(theme.statusWaitingInput, theme.statusColor("waiting_for_input"))
        assertEquals(theme.statusWaitingPermission, theme.statusColor("waiting_for_permission"))
        assertEquals(theme.statusIdle, theme.statusColor("idle"))
        assertEquals(theme.statusEnded, theme.statusColor("ended"))
    }

    @Test
    fun `statusColor returns gray for unknown status`() {
        assertEquals(Color.Gray, theme.statusColor("unknown_status"))
        assertEquals(Color.Gray, theme.statusColor(""))
    }

    @Test
    fun `platformColor returns correct color for platforms`() {
        assertEquals(theme.platformMac, theme.platformColor("mac"))
        assertEquals(theme.platformMac, theme.platformColor("macos"))
        assertEquals(theme.platformMac, theme.platformColor("darwin"))
        assertEquals(theme.platformLinux, theme.platformColor("linux"))
        assertEquals(theme.platformWindows, theme.platformColor("windows"))
    }

    @Test
    fun `platformColor is case insensitive`() {
        assertEquals(theme.platformMac, theme.platformColor("MAC"))
        assertEquals(theme.platformLinux, theme.platformColor("Linux"))
        assertEquals(theme.platformWindows, theme.platformColor("WINDOWS"))
        assertEquals(theme.platformMac, theme.platformColor("DaRwIn"))
    }

    @Test
    fun `platformColor returns default for unknown platform`() {
        assertEquals(theme.platformDefault, theme.platformColor("unknown"))
        assertEquals(theme.platformDefault, theme.platformColor(""))
    }

    @Test
    fun `eventColor returns correct color for known events`() {
        assertEquals(theme.eventSessionStart, theme.eventColor("SessionStart"))
        assertEquals(theme.eventSessionEnd, theme.eventColor("SessionEnd"))
        assertEquals(theme.eventStop, theme.eventColor("Stop"))
        assertEquals(theme.eventNotification, theme.eventColor("Notification"))
        assertEquals(theme.eventUserPromptSubmit, theme.eventColor("UserPromptSubmit"))
    }

    @Test
    fun `eventColor returns default for unknown event`() {
        assertEquals(theme.eventDefault, theme.eventColor("UnknownEvent"))
        assertEquals(theme.eventDefault, theme.eventColor(""))
    }

    @Test
    fun `eventColor is case sensitive`() {
        assertEquals(theme.eventSessionStart, theme.eventColor("SessionStart"))
        assertEquals(theme.eventDefault, theme.eventColor("sessionstart"))
    }

    @Test
    fun `constants have expected values`() {
        assertEquals(12f, AppThemeDefaults.cardCornerRadius)
        assertEquals(0.3f, AppThemeDefaults.cardBorderOpacity)
        assertEquals(0.5f, AppThemeDefaults.cardBorderWidth)
    }

    @Test
    fun `all themes list contains 4 themes`() {
        assertEquals(4, allThemes.size)
        assertEquals("standard", allThemes[0].id)
        assertEquals("neon_ops", allThemes[1].id)
        assertEquals("solarized", allThemes[2].id)
        assertEquals("arctic", allThemes[3].id)
    }

    @Test
    fun `each theme has 4 preview colors`() {
        for (t in allThemes) {
            assertEquals("${t.name} should have 4 preview colors", 4, t.preview.size)
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd android && ./gradlew test
```

Expected: compilation error.

- [ ] **Step 3: Create AppTheme.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/ui/theme/AppTheme.kt
package com.claudiator.app.ui.theme

import androidx.compose.ui.graphics.Color

interface AppTheme {
    val id: String
    val name: String
    val preview: List<Color>

    // Status
    val statusActive: Color
    val statusWaitingInput: Color
    val statusWaitingPermission: Color
    val statusIdle: Color
    val statusEnded: Color

    // Platform
    val platformMac: Color
    val platformLinux: Color
    val platformWindows: Color
    val platformDefault: Color

    // Event
    val eventSessionStart: Color
    val eventSessionEnd: Color
    val eventStop: Color
    val eventNotification: Color
    val eventUserPromptSubmit: Color
    val eventDefault: Color

    // Server
    val serverConnected: Color
    val serverDisconnected: Color

    // UI
    val uiError: Color
    val uiAccent: Color
    val uiTint: Color

    // Surface (light/dark)
    fun pageBackground(isDark: Boolean): Color
    fun cardBackground(isDark: Boolean): Color
    fun cardBorder(isDark: Boolean): Color
}

fun AppTheme.statusColor(status: String): Color = when (status) {
    "active" -> statusActive
    "waiting_for_input" -> statusWaitingInput
    "waiting_for_permission" -> statusWaitingPermission
    "idle" -> statusIdle
    "ended" -> statusEnded
    else -> Color.Gray
}

fun AppTheme.platformColor(platform: String): Color = when (platform.lowercase()) {
    "mac", "macos", "darwin" -> platformMac
    "linux" -> platformLinux
    "windows" -> platformWindows
    else -> platformDefault
}

fun AppTheme.eventColor(hookEventName: String): Color = when (hookEventName) {
    "SessionStart" -> eventSessionStart
    "SessionEnd" -> eventSessionEnd
    "Stop" -> eventStop
    "Notification" -> eventNotification
    "UserPromptSubmit" -> eventUserPromptSubmit
    else -> eventDefault
}

object AppThemeDefaults {
    const val cardCornerRadius = 12f
    const val cardBorderOpacity = 0.3f
    const val cardBorderWidth = 0.5f
}
```

- [ ] **Step 4: Create Themes.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/ui/theme/Themes.kt
package com.claudiator.app.ui.theme

import androidx.compose.ui.graphics.Color

val allThemes: List<AppTheme> = listOf(StandardTheme, NeonOpsTheme, SolarizedTheme, ArcticTheme)

object StandardTheme : AppTheme {
    override val id = "standard"
    override val name = "Standard"
    override val preview = listOf(Color(0xFF4CAF50), Color(0xFF2196F3), Color(0xFFFF9800), Color(0xFFF44336))

    override val statusActive = Color(0xFF4CAF50)
    override val statusWaitingInput = Color(0xFFFF9800)
    override val statusWaitingPermission = Color(0xFFF44336)
    override val statusIdle = Color.Gray
    override val statusEnded = Color.Gray

    override val platformMac = Color(0xFF2196F3)
    override val platformLinux = Color(0xFFFF9800)
    override val platformWindows = Color(0xFF00BCD4)
    override val platformDefault = Color.Gray

    override val eventSessionStart = Color(0xFF4CAF50)
    override val eventSessionEnd = Color.Gray
    override val eventStop = Color(0xFFFF9800)
    override val eventNotification = Color(0xFFF44336)
    override val eventUserPromptSubmit = Color(0xFF2196F3)
    override val eventDefault = Color.Gray

    override val serverConnected = Color(0xFF4CAF50)
    override val serverDisconnected = Color(0xFFF44336)

    override val uiError = Color(0xFFF44336)
    override val uiAccent = Color(0xFF2196F3)
    override val uiTint = Color(0xFF2196F3)

    override fun pageBackground(isDark: Boolean) = if (isDark) Color(0xFF121212) else Color(0xFFF2F2F7)
    override fun cardBackground(isDark: Boolean) = if (isDark) Color(0xFF1C1C1E) else Color(0xFFFFFFFF)
    override fun cardBorder(isDark: Boolean) = if (isDark) Color(0xFF333333) else Color(0xFFCCCCCC)
}

object NeonOpsTheme : AppTheme {
    override val id = "neon_ops"
    override val name = "Neon Ops"
    override val preview = listOf(Color(0xFF00FF99), Color(0xFFFF4080), Color(0xFFFFC200), Color(0xFF00BFFF))

    override val statusActive = Color(0xFF00FF99)
    override val statusWaitingInput = Color(0xFFFFC200)
    override val statusWaitingPermission = Color(0xFFFF4080)
    override val statusIdle = Color(0xFF888888)
    override val statusEnded = Color(0xFF666666)

    override val platformMac = Color(0xFF00BFFF)
    override val platformLinux = Color(0xFFFFC200)
    override val platformWindows = Color(0xFF00FF99)
    override val platformDefault = Color(0xFF888888)

    override val eventSessionStart = Color(0xFF00FF99)
    override val eventSessionEnd = Color(0xFF666666)
    override val eventStop = Color(0xFFFFC200)
    override val eventNotification = Color(0xFFFF4080)
    override val eventUserPromptSubmit = Color(0xFF00BFFF)
    override val eventDefault = Color(0xFF888888)

    override val serverConnected = Color(0xFF00FF99)
    override val serverDisconnected = Color(0xFFFF4080)

    override val uiError = Color(0xFFFF4080)
    override val uiAccent = Color(0xFF00BFFF)
    override val uiTint = Color(0xFF00FF99)

    override fun pageBackground(isDark: Boolean) = if (isDark) Color(0xFF050D08) else Color(0xFFE0F2E9)
    override fun cardBackground(isDark: Boolean) = if (isDark) Color(0xFF0F1F14) else Color(0xFFF8FFF9)
    override fun cardBorder(isDark: Boolean) = if (isDark) Color(0xFF009959) else Color(0xFF00CC80)
}

object SolarizedTheme : AppTheme {
    override val id = "solarized"
    override val name = "Solarized"
    override val preview = listOf(Color(0xFF859900), Color(0xFF268BD2), Color(0xFFB58900), Color(0xFFDC322F))

    override val statusActive = Color(0xFF859900)
    override val statusWaitingInput = Color(0xFFB58900)
    override val statusWaitingPermission = Color(0xFFDC322F)
    override val statusIdle = Color(0xFF93A1A1)
    override val statusEnded = Color(0xFF839496)

    override val platformMac = Color(0xFF268BD2)
    override val platformLinux = Color(0xFFCB4B16)
    override val platformWindows = Color(0xFF2AA198)
    override val platformDefault = Color(0xFF93A1A1)

    override val eventSessionStart = Color(0xFF859900)
    override val eventSessionEnd = Color(0xFF839496)
    override val eventStop = Color(0xFFCB4B16)
    override val eventNotification = Color(0xFFDC322F)
    override val eventUserPromptSubmit = Color(0xFF268BD2)
    override val eventDefault = Color(0xFF93A1A1)

    override val serverConnected = Color(0xFF859900)
    override val serverDisconnected = Color(0xFFDC322F)

    override val uiError = Color(0xFFDC322F)
    override val uiAccent = Color(0xFF268BD2)
    override val uiTint = Color(0xFF268BD2)

    override fun pageBackground(isDark: Boolean) = if (isDark) Color(0xFF002129) else Color(0xFFE3DEC9)
    override fun cardBackground(isDark: Boolean) = if (isDark) Color(0xFF083642) else Color(0xFFFAF5E3)
    override fun cardBorder(isDark: Boolean) = if (isDark) Color(0xFF124A57) else Color(0xFFD1C7B0)
}

object ArcticTheme : AppTheme {
    override val id = "arctic"
    override val name = "Arctic"
    override val preview = listOf(Color(0xFF2EB887), Color(0xFF3B82F6), Color(0xFF8B95A5), Color(0xFFEF4444))

    override val statusActive = Color(0xFF2EB887)
    override val statusWaitingInput = Color(0xFFF59E0B)
    override val statusWaitingPermission = Color(0xFFEF4444)
    override val statusIdle = Color(0xFF8B95A5)
    override val statusEnded = Color(0xFF6B7280)

    override val platformMac = Color(0xFF3B82F6)
    override val platformLinux = Color(0xFFF59E0B)
    override val platformWindows = Color(0xFF06B6D4)
    override val platformDefault = Color(0xFF8B95A5)

    override val eventSessionStart = Color(0xFF2EB887)
    override val eventSessionEnd = Color(0xFF6B7280)
    override val eventStop = Color(0xFFF59E0B)
    override val eventNotification = Color(0xFFEF4444)
    override val eventUserPromptSubmit = Color(0xFF3B82F6)
    override val eventDefault = Color(0xFF8B95A5)

    override val serverConnected = Color(0xFF2EB887)
    override val serverDisconnected = Color(0xFFEF4444)

    override val uiError = Color(0xFFEF4444)
    override val uiAccent = Color(0xFF3B82F6)
    override val uiTint = Color(0xFF2EB887)

    override fun pageBackground(isDark: Boolean) = if (isDark) Color(0xFF080F1A) else Color(0xFFDEEBFA)
    override fun cardBackground(isDark: Boolean) = if (isDark) Color(0xFF142133) else Color(0xFFF5FAFF)
    override fun cardBorder(isDark: Boolean) = if (isDark) Color(0xFF213045) else Color(0xFFC7DBEE)
}
```

- [ ] **Step 5: Create ThemeManager.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/ui/theme/ThemeManager.kt
package com.claudiator.app.ui.theme

import android.content.Context
import android.content.SharedPreferences
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

enum class AppearanceMode { SYSTEM, LIGHT, DARK }

class ThemeManager(context: Context) {

    private val prefs: SharedPreferences =
        context.getSharedPreferences("claudiator_theme", Context.MODE_PRIVATE)

    private val _currentTheme = MutableStateFlow(loadTheme())
    val currentTheme: StateFlow<AppTheme> = _currentTheme.asStateFlow()

    private val _appearanceMode = MutableStateFlow(loadAppearance())
    val appearanceMode: StateFlow<AppearanceMode> = _appearanceMode.asStateFlow()

    fun selectTheme(theme: AppTheme) {
        prefs.edit().putString("theme_id", theme.id).apply()
        _currentTheme.value = theme
    }

    fun setAppearance(mode: AppearanceMode) {
        prefs.edit().putString("appearance_mode", mode.name).apply()
        _appearanceMode.value = mode
    }

    private fun loadTheme(): AppTheme {
        val savedId = prefs.getString("theme_id", "standard")
        return allThemes.firstOrNull { it.id == savedId } ?: StandardTheme
    }

    private fun loadAppearance(): AppearanceMode {
        val saved = prefs.getString("appearance_mode", "SYSTEM")
        return try {
            AppearanceMode.valueOf(saved ?: "SYSTEM")
        } catch (_: Exception) {
            AppearanceMode.SYSTEM
        }
    }
}
```

- [ ] **Step 6: Create ClaudiatorTheme.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/ui/theme/ClaudiatorTheme.kt
package com.claudiator.app.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue

val LocalAppTheme = compositionLocalOf<AppTheme> { StandardTheme }
val LocalIsDarkTheme = compositionLocalOf { false }

@Composable
fun ClaudiatorTheme(
    themeManager: ThemeManager,
    content: @Composable () -> Unit,
) {
    val theme by themeManager.currentTheme.collectAsState()
    val mode by themeManager.appearanceMode.collectAsState()

    val isDark = when (mode) {
        AppearanceMode.SYSTEM -> isSystemInDarkTheme()
        AppearanceMode.LIGHT -> false
        AppearanceMode.DARK -> true
    }

    MaterialTheme(
        colorScheme = if (isDark) darkColorScheme() else lightColorScheme(),
    ) {
        CompositionLocalProvider(
            LocalAppTheme provides theme,
            LocalIsDarkTheme provides isDark,
        ) {
            content()
        }
    }
}
```

- [ ] **Step 7: Run tests to verify they pass**

```bash
cd android && ./gradlew test
```

Expected: all ThemeTest tests PASS.

- [ ] **Step 8: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/ui/theme/ android/app/src/test/java/com/claudiator/app/ThemeTest.kt
git commit -m "add theme system with 4 themes"
```

---

### Task 5: SecureStorage & ApiClient

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/services/SecureStorage.kt`
- Create: `android/app/src/main/java/com/claudiator/app/services/ApiClient.kt`

- [ ] **Step 1: Create SecureStorage.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/services/SecureStorage.kt
package com.claudiator.app.services

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKeys

class SecureStorage(context: Context) {

    private val prefs: SharedPreferences = EncryptedSharedPreferences.create(
        "claudiator_secure_prefs",
        MasterKeys.getOrCreate(MasterKeys.AES256_GCM_SPEC),
        context,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
    )

    var serverUrl: String?
        get() = prefs.getString("server_url", null)
        set(value) = prefs.edit().putString("server_url", value).apply()

    var apiKey: String?
        get() = prefs.getString("api_key", null)
        set(value) = prefs.edit().putString("api_key", value).apply()

    val isConfigured: Boolean
        get() = !serverUrl.isNullOrEmpty() && !apiKey.isNullOrEmpty()

    fun clear() {
        prefs.edit().clear().apply()
    }
}
```

- [ ] **Step 2: Create ApiClient.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/services/ApiClient.kt
package com.claudiator.app.services

import com.claudiator.app.models.*
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.engine.android.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.json.Json

sealed class ApiError(message: String) : Exception(message) {
    object NotConfigured : ApiError("Server not configured")
    object InvalidURL : ApiError("Invalid server URL")
    object Unauthorized : ApiError("Invalid API key")
    data class ServerError(val code: Int) : ApiError("Server error ($code)")
    data class NetworkError(override val cause: Throwable) : ApiError(cause.message ?: "Network error")
    data class DecodingError(override val cause: Throwable) : ApiError("Data error: ${cause.message}")
}

class ApiClient(private val secureStorage: SecureStorage) {

    private val json = Json { ignoreUnknownKeys = true }

    private val client = HttpClient(Android) {
        install(ContentNegotiation) {
            json(json)
        }
        install(HttpTimeout) {
            requestTimeoutMillis = 15_000
            connectTimeoutMillis = 15_000
            socketTimeoutMillis = 30_000
        }
    }

    private val _isConfigured = MutableStateFlow(secureStorage.isConfigured)
    val isConfigured: StateFlow<Boolean> = _isConfigured.asStateFlow()

    fun refreshConfigState() {
        _isConfigured.value = secureStorage.isConfigured
    }

    private val retryableExceptions = setOf(
        "connect", "timeout", "timed out", "network", "refused",
    )

    private suspend inline fun <reified T> request(
        path: String,
        method: HttpMethod = HttpMethod.Get,
        body: Any? = null,
    ): T {
        val baseUrl = secureStorage.serverUrl
        val apiKey = secureStorage.apiKey
        if (baseUrl.isNullOrEmpty() || apiKey.isNullOrEmpty()) throw ApiError.NotConfigured

        val urlString = if (baseUrl.endsWith("/")) baseUrl + path.removePrefix("/") else baseUrl + path

        var lastError: Exception? = null
        for (attempt in 0 until 3) {
            if (attempt > 0) delay(500L * attempt)
            try {
                val response = client.request(urlString) {
                    this.method = method
                    header("Authorization", "Bearer $apiKey")
                    contentType(ContentType.Application.Json)
                    if (body != null) setBody(body)
                }
                when (response.status.value) {
                    in 200..299 -> return response.body<T>()
                    401 -> throw ApiError.Unauthorized
                    else -> throw ApiError.ServerError(response.status.value)
                }
            } catch (e: ApiError) {
                throw e
            } catch (e: Exception) {
                val msg = (e.message ?: "").lowercase()
                if (retryableExceptions.any { msg.contains(it) }) {
                    lastError = e
                } else {
                    throw ApiError.NetworkError(e)
                }
            }
        }
        throw ApiError.NetworkError(lastError ?: Exception("Unknown error"))
    }

    suspend fun ping(): PingResponse = request("/api/v1/ping")

    suspend fun fetchDevices(): List<Device> =
        request<DevicesResponse>("/api/v1/devices").devices

    suspend fun fetchSessions(deviceId: String, status: String? = null, limit: Int? = null): List<Session> {
        val params = buildList {
            if (status != null) add("status=$status")
            if (limit != null) add("limit=$limit")
        }
        val query = if (params.isNotEmpty()) "?" + params.joinToString("&") else ""
        return request<SessionsResponse>("/api/v1/devices/$deviceId/sessions$query").sessions
    }

    suspend fun fetchAllSessions(status: String? = null, limit: Int? = null): List<Session> {
        val params = buildList {
            if (status != null) add("status=$status")
            if (limit != null) add("limit=$limit")
        }
        val query = if (params.isNotEmpty()) "?" + params.joinToString("&") else ""
        return request<SessionsResponse>("/api/v1/sessions$query").sessions
    }

    suspend fun fetchAllSessionsPage(
        excludeEnded: Boolean = false,
        limit: Int = 50,
        offset: Int = 0,
    ): SessionListPage {
        val params = buildList {
            if (excludeEnded) add("exclude_ended=true")
            add("limit=$limit")
            add("offset=$offset")
        }
        return request("/api/v1/sessions?" + params.joinToString("&"))
    }

    suspend fun fetchEvents(sessionId: String, limit: Int? = null): List<Event> {
        val query = if (limit != null) "?limit=$limit" else ""
        return request<EventsResponse>("/api/v1/sessions/$sessionId/events$query").events
    }

    suspend fun registerPushToken(token: String) {
        @kotlinx.serialization.Serializable
        data class Body(
            val platform: String,
            @kotlinx.serialization.SerialName("push_token") val pushToken: String,
            val sandbox: Boolean,
        )
        request<Map<String, String>>(
            "/api/v1/push/register",
            method = HttpMethod.Post,
            body = Body(platform = "android", pushToken = token, sandbox = false),
        )
    }

    suspend fun fetchNotifications(after: String? = null, limit: Int? = null): List<AppNotification> {
        val params = buildList {
            if (after != null) add("after=$after")
            if (limit != null) add("limit=$limit")
        }
        val query = if (params.isNotEmpty()) "?" + params.joinToString("&") else ""
        return request<NotificationsResponse>("/api/v1/notifications$query").notifications
    }

    suspend fun acknowledgeNotifications(ids: List<String>) {
        @kotlinx.serialization.Serializable
        data class Body(@kotlinx.serialization.SerialName("notification_ids") val notificationIds: List<String>)
        request<Map<String, String>>(
            "/api/v1/notifications/ack",
            method = HttpMethod.Post,
            body = Body(notificationIds = ids),
        )
    }

    fun configure(url: String, apiKey: String) {
        secureStorage.serverUrl = url
        secureStorage.apiKey = apiKey
        refreshConfigState()
    }

    fun disconnect() {
        secureStorage.clear()
        refreshConfigState()
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/services/SecureStorage.kt android/app/src/main/java/com/claudiator/app/services/ApiClient.kt
git commit -m "add SecureStorage and ApiClient services"
```

---

### Task 6: AppNotificationManager & VersionMonitor

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/services/AppNotificationManager.kt`
- Create: `android/app/src/main/java/com/claudiator/app/services/VersionMonitor.kt`
- Create: `android/app/src/test/java/com/claudiator/app/NotificationManagerTest.kt`

- [ ] **Step 1: Write NotificationManagerTest.kt**

```kotlin
// android/app/src/test/java/com/claudiator/app/NotificationManagerTest.kt
package com.claudiator.app

import com.claudiator.app.models.AppNotification
import com.claudiator.app.services.AppNotificationManager
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test

class NotificationManagerTest {

    private lateinit var manager: AppNotificationManager

    private fun notif(id: String, sessionId: String = "sess1") = AppNotification(
        notificationId = id,
        sessionId = sessionId,
        deviceId = "dev1",
        title = "Test",
        body = "Test body",
        notificationType = "info",
        createdAt = "2024-01-15T10:00:00Z",
    )

    @Before
    fun setUp() {
        manager = AppNotificationManager()
    }

    @Test
    fun `initializes with empty state`() {
        assertEquals(0, manager.state.value.unreadCount)
        assertTrue(manager.state.value.notifications.isEmpty())
        assertTrue(manager.state.value.unreadSessionIds.isEmpty())
    }

    @Test
    fun `processNotifications adds unread notifications`() {
        manager.processNotifications(listOf(notif("n1"), notif("n2")))
        assertEquals(2, manager.state.value.unreadCount)
        assertEquals(2, manager.state.value.notifications.size)
    }

    @Test
    fun `markSessionRead removes notifications for session`() {
        manager.processNotifications(listOf(
            notif("n1", "sess1"),
            notif("n2", "sess1"),
            notif("n3", "sess2"),
        ))
        val acked = manager.markSessionRead("sess1")
        assertEquals(listOf("n1", "n2"), acked)
        assertEquals(1, manager.state.value.unreadCount)
        assertFalse(manager.state.value.unreadSessionIds.contains("sess1"))
        assertTrue(manager.state.value.unreadSessionIds.contains("sess2"))
    }

    @Test
    fun `markSessionRead handles non-existent session`() {
        manager.processNotifications(listOf(notif("n1", "sess1")))
        val acked = manager.markSessionRead("sess99")
        assertTrue(acked.isEmpty())
        assertEquals(1, manager.state.value.unreadCount)
    }

    @Test
    fun `markNotificationRead removes specific notification`() {
        manager.processNotifications(listOf(notif("n1"), notif("n2")))
        val acked = manager.markNotificationRead("n1")
        assertEquals("n1", acked)
        assertEquals(1, manager.state.value.unreadCount)
    }

    @Test
    fun `markNotificationRead returns null for non-existent`() {
        manager.processNotifications(listOf(notif("n1")))
        assertNull(manager.markNotificationRead("n99"))
        assertEquals(1, manager.state.value.unreadCount)
    }

    @Test
    fun `markAllRead clears all unread`() {
        manager.processNotifications(listOf(notif("n1"), notif("n2"), notif("n3")))
        val acked = manager.markAllRead()
        assertEquals(3, acked.size)
        assertEquals(0, manager.state.value.unreadCount)
        assertTrue(manager.state.value.unreadSessionIds.isEmpty())
    }

    @Test
    fun `markAllRead returns empty when no unread`() {
        assertTrue(manager.markAllRead().isEmpty())
    }

    @Test
    fun `duplicate notifications are not added`() {
        manager.processNotifications(listOf(notif("n1")))
        manager.processNotifications(listOf(notif("n1")))
        assertEquals(1, manager.state.value.notifications.size)
    }

    @Test
    fun `caps at 100 notifications`() {
        val notifs = (1..120).map { notif("n$it") }
        manager.processNotifications(notifs)
        assertEquals(100, manager.state.value.notifications.size)
    }

    @Test
    fun `markReceivedViaPush tracks push-received IDs`() {
        manager.markReceivedViaPush("n1")
        assertTrue(manager.isPushReceived("n1"))
        assertFalse(manager.isPushReceived("n2"))
    }

    @Test
    fun `acknowledged notifications are marked read`() {
        val acked = notif("n1").copy(acknowledged = true)
        manager.processNotifications(listOf(acked, notif("n2")))
        assertEquals(1, manager.state.value.unreadCount) // only n2 is unread
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd android && ./gradlew test
```

Expected: compilation error.

- [ ] **Step 3: Create AppNotificationManager.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/services/AppNotificationManager.kt
package com.claudiator.app.services

import com.claudiator.app.models.AppNotification
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

data class NotificationState(
    val notifications: List<AppNotification> = emptyList(),
    val unreadCount: Int = 0,
    val unreadSessionIds: Set<String> = emptySet(),
    val readIds: Set<String> = emptySet(),
)

class AppNotificationManager {

    private val _state = MutableStateFlow(NotificationState())
    val state: StateFlow<NotificationState> = _state.asStateFlow()

    private val pushReceivedIds = mutableMapOf<String, Long>() // id -> timestamp
    private val pushRetentionMs = 10 * 60 * 1000L // 10 minutes

    private val maxNotifications = 100
    private val maxReadIds = 500

    var lastSeenId: String? = null
        private set

    fun processNotifications(incoming: List<AppNotification>) {
        if (incoming.isEmpty()) return

        // Update lastSeen to newest (last in ascending list)
        incoming.lastOrNull()?.let { lastSeenId = it.createdAt }

        _state.update { current ->
            val existingIds = current.notifications.map { it.notificationId }.toSet()
            val newNotifs = incoming.filter { it.notificationId !in existingIds }

            val allNotifs = (newNotifs + current.notifications).take(maxNotifications)

            // Seed read state from server acknowledged
            val serverAckedIds = incoming
                .filter { it.acknowledged == true }
                .map { it.notificationId }
                .toSet()
            val readIds = (current.readIds + serverAckedIds).let { ids ->
                if (ids.size > maxReadIds) ids.sorted().takeLast(maxReadIds).toSet() else ids
            }

            val unread = allNotifs.filter { it.notificationId !in readIds }

            current.copy(
                notifications = allNotifs,
                readIds = readIds,
                unreadCount = unread.size,
                unreadSessionIds = unread.map { it.sessionId }.toSet(),
            )
        }
    }

    /** Returns list of notification IDs that were marked read (for server ack). */
    fun markSessionRead(sessionId: String): List<String> {
        var markedIds = emptyList<String>()
        _state.update { current ->
            val toMark = current.notifications
                .filter { it.sessionId == sessionId && it.notificationId !in current.readIds }
                .map { it.notificationId }
            if (toMark.isEmpty()) return@update current

            markedIds = toMark
            val readIds = trimReadIds(current.readIds + toMark)
            val unread = current.notifications.filter { it.notificationId !in readIds }

            current.copy(
                readIds = readIds,
                unreadCount = unread.size,
                unreadSessionIds = unread.map { it.sessionId }.toSet(),
            )
        }
        return markedIds
    }

    /** Returns the notification ID if it was marked read, null otherwise. */
    fun markNotificationRead(notificationId: String): String? {
        var marked: String? = null
        _state.update { current ->
            if (notificationId in current.readIds ||
                current.notifications.none { it.notificationId == notificationId }
            ) {
                return@update current
            }

            marked = notificationId
            val readIds = trimReadIds(current.readIds + notificationId)
            val unread = current.notifications.filter { it.notificationId !in readIds }

            current.copy(
                readIds = readIds,
                unreadCount = unread.size,
                unreadSessionIds = unread.map { it.sessionId }.toSet(),
            )
        }
        return marked
    }

    /** Returns list of all notification IDs that were marked read. */
    fun markAllRead(): List<String> {
        var markedIds = emptyList<String>()
        _state.update { current ->
            val unreadIds = current.notifications
                .filter { it.notificationId !in current.readIds }
                .map { it.notificationId }
            if (unreadIds.isEmpty()) return@update current

            markedIds = unreadIds
            val readIds = trimReadIds(current.readIds + unreadIds)

            current.copy(
                readIds = readIds,
                unreadCount = 0,
                unreadSessionIds = emptySet(),
            )
        }
        return markedIds
    }

    fun markReceivedViaPush(notificationId: String) {
        cleanupOldPushIds()
        pushReceivedIds[notificationId] = System.currentTimeMillis()
    }

    fun isPushReceived(notificationId: String): Boolean {
        cleanupOldPushIds()
        return notificationId in pushReceivedIds
    }

    private fun cleanupOldPushIds() {
        val cutoff = System.currentTimeMillis() - pushRetentionMs
        pushReceivedIds.entries.removeAll { it.value < cutoff }
    }

    private fun trimReadIds(ids: Set<String>): Set<String> {
        return if (ids.size > maxReadIds) ids.sorted().takeLast(maxReadIds).toSet() else ids
    }
}
```

- [ ] **Step 4: Create VersionMonitor.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/services/VersionMonitor.kt
package com.claudiator.app.services

import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class VersionMonitor(
    private val apiClient: ApiClient,
    private val notificationManager: AppNotificationManager,
) {
    private val _dataVersion = MutableStateFlow(0L)
    val dataVersion: StateFlow<Long> = _dataVersion.asStateFlow()

    private var notificationVersion = 0L
    private var pollingJob: Job? = null

    fun start(scope: CoroutineScope) {
        if (pollingJob != null) return
        pollingJob = scope.launch {
            while (isActive) {
                try {
                    val ping = apiClient.ping()
                    _dataVersion.value = ping.dataVersion
                    if (ping.notificationVersion != notificationVersion) {
                        notificationVersion = ping.notificationVersion
                        val lastSeen = notificationManager.lastSeenId
                        val notifications = apiClient.fetchNotifications(after = lastSeen)
                        notificationManager.processNotifications(notifications)
                    }
                } catch (_: Exception) {
                    // silently retry next cycle
                }
                delay(10_000)
            }
        }
    }

    fun stop() {
        pollingJob?.cancel()
        pollingJob = null
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cd android && ./gradlew test
```

Expected: all NotificationManagerTest tests PASS.

- [ ] **Step 6: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/services/AppNotificationManager.kt android/app/src/main/java/com/claudiator/app/services/VersionMonitor.kt android/app/src/test/java/com/claudiator/app/NotificationManagerTest.kt
git commit -m "add notification manager and version monitor"
```

---

### Task 7: ViewModels

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/viewmodels/SetupViewModel.kt`
- Create: `android/app/src/main/java/com/claudiator/app/viewmodels/DeviceListViewModel.kt`
- Create: `android/app/src/main/java/com/claudiator/app/viewmodels/SessionListViewModel.kt`
- Create: `android/app/src/main/java/com/claudiator/app/viewmodels/AllSessionsViewModel.kt`
- Create: `android/app/src/main/java/com/claudiator/app/viewmodels/EventListViewModel.kt`
- Create: `android/app/src/test/java/com/claudiator/app/ViewModelTest.kt`
- Create: `android/app/src/test/java/com/claudiator/app/SetupViewModelTest.kt`

This task creates all 5 ViewModels plus their tests. The ViewModels are direct ports of the iOS ones, using `StateFlow` instead of `@Observable`.

Due to the length, the full ViewModel code and tests follow the exact same patterns as the iOS versions but using Kotlin coroutines and StateFlow. Each ViewModel:
- Holds a `MutableStateFlow<UiState>` with the screen's state
- Exposes it as `StateFlow` via `.asStateFlow()`
- Has `refresh()` and other methods that launch coroutines in `viewModelScope`
- Mirrors the exact filtering, grouping, and pagination logic from iOS

**Implementation instructions for each ViewModel:**

- [ ] **Step 1: Create SetupViewModel.kt** — Port `SetupViewModel.swift`. State: `serverUrl`, `apiKey`, `isLoading`, `error`, `connectionSuccess`. `connect()` validates URL via `URLValidator`, tests via `apiClient.ping()`, saves on success.

- [ ] **Step 2: Create DeviceListViewModel.kt** — Port `DeviceListViewModel.swift` with `SessionStatusCounts`. `refresh()` fetches devices + all sessions, aggregates counts per device.

- [ ] **Step 3: Create SessionListViewModel.kt** — Port `SessionListViewModel.swift`. Holds `deviceId`, `filter` (active/all). `refresh()` fetches sessions for device, client-side filters for active.

- [ ] **Step 4: Create AllSessionsViewModel.kt** — Port `AllSessionsViewModel.swift`. Pagination (`currentOffset`, `hasMore`), grouping (`isGroupedByDevice`, `expandedDevices`, `groupedSessions`), `loadMore()` with deduplication.

- [ ] **Step 5: Create EventListViewModel.kt** — Port `EventListViewModel.swift`. Holds `sessionId`. `refresh()` fetches events.

- [ ] **Step 6: Write ViewModelTest.kt** — Port `ViewModelTests.swift`. Test initialization defaults, `SessionStatusCounts`, grouping toggle, device toggle, filter enums, session grouping logic, status count aggregation.

- [ ] **Step 7: Write SetupViewModelTest.kt** — Port `SetupViewModelTests.swift`. Test initial state, URL cleaning logic, error message format verification.

- [ ] **Step 8: Run tests**

```bash
cd android && ./gradlew test
```

Expected: all ViewModel tests PASS.

- [ ] **Step 9: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/viewmodels/ android/app/src/test/java/com/claudiator/app/ViewModelTest.kt android/app/src/test/java/com/claudiator/app/SetupViewModelTest.kt
git commit -m "add viewmodels with tests"
```

---

### Task 8: FCM Service

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/services/FcmService.kt`

- [ ] **Step 1: Create FcmService.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/services/FcmService.kt
package com.claudiator.app.services

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.core.app.NotificationCompat
import com.claudiator.app.ClaudiatorApp
import com.claudiator.app.MainActivity
import com.claudiator.app.R
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

class FcmService : FirebaseMessagingService() {

    private val serviceScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    override fun onNewToken(token: String) {
        // Store token for later registration
        getSharedPreferences("claudiator_fcm", Context.MODE_PRIVATE)
            .edit()
            .putString("fcm_token", token)
            .apply()

        // If API client is configured, register immediately
        val app = application as? ClaudiatorApp ?: return
        if (app.secureStorage.isConfigured) {
            serviceScope.launch {
                try {
                    app.apiClient.registerPushToken(token)
                } catch (_: Exception) {
                    // Will retry on next app launch
                }
            }
        }
    }

    override fun onMessageReceived(message: RemoteMessage) {
        val data = message.data
        val notificationId = data["notification_id"] ?: return
        val sessionId = data["session_id"] ?: return
        val title = data["title"] ?: "Claudiator"
        val body = data["body"] ?: ""

        val app = application as? ClaudiatorApp ?: return

        // Mark as received via push (for deduplication)
        app.notificationManager.markReceivedViaPush(notificationId)

        // Show Android notification
        showNotification(notificationId, sessionId, title, body)
    }

    private fun showNotification(notificationId: String, sessionId: String, title: String, body: String) {
        val notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

        // Ensure channel exists
        val channel = NotificationChannel(
            CHANNEL_ID,
            getString(R.string.notification_channel_name),
            NotificationManager.IMPORTANCE_HIGH,
        ).apply {
            description = getString(R.string.notification_channel_description)
            enableVibration(true)
        }
        notificationManager.createNotificationChannel(channel)

        // Deep link intent
        val intent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP
            putExtra("session_id", sessionId)
            putExtra("notification_id", notificationId)
        }
        val pendingIntent = PendingIntent.getActivity(
            this, notificationId.hashCode(), intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )

        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_launcher_foreground)
            .setContentTitle(title)
            .setContentText(body)
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setAutoCancel(true)
            .setContentIntent(pendingIntent)
            .build()

        notificationManager.notify(notificationId.hashCode(), notification)
    }

    companion object {
        const val CHANNEL_ID = "claudiator_sessions"

        fun getStoredToken(context: Context): String? =
            context.getSharedPreferences("claudiator_fcm", Context.MODE_PRIVATE)
                .getString("fcm_token", null)
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/services/FcmService.kt
git commit -m "add FCM push notification service"
```

---

### Task 9: Application Class & Navigation

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/ClaudiatorApp.kt`
- Create: `android/app/src/main/java/com/claudiator/app/MainActivity.kt`
- Create: `android/app/src/main/java/com/claudiator/app/navigation/Screen.kt`
- Create: `android/app/src/main/java/com/claudiator/app/navigation/AppNavigation.kt`

- [ ] **Step 1: Create ClaudiatorApp.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/ClaudiatorApp.kt
package com.claudiator.app

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager
import com.claudiator.app.services.*
import com.claudiator.app.services.FcmService
import com.claudiator.app.ui.theme.ThemeManager

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
        notificationManager = AppNotificationManager()
        versionMonitor = VersionMonitor(apiClient, notificationManager)
        themeManager = ThemeManager(this)

        createNotificationChannel()
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            FcmService.CHANNEL_ID,
            getString(R.string.notification_channel_name),
            NotificationManager.IMPORTANCE_HIGH,
        ).apply {
            description = getString(R.string.notification_channel_description)
            enableVibration(true)
        }
        val manager = getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(channel)
    }
}
```

- [ ] **Step 2: Create Screen.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/navigation/Screen.kt
package com.claudiator.app.navigation

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

- [ ] **Step 3: Create AppNavigation.kt**

This file contains:
- `AppNavigation` composable with `NavHost` routing (Setup vs Main)
- `MainScaffold` composable with bottom navigation bar (3 tabs) + `HorizontalPager` for swipe
- Each tab has its own `NavHost` for push/pop navigation

```kotlin
// android/app/src/main/java/com/claudiator/app/navigation/AppNavigation.kt
package com.claudiator.app.navigation

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavHostController
import androidx.navigation.compose.*
import com.claudiator.app.ClaudiatorApp
import com.claudiator.app.services.*
import com.claudiator.app.ui.devices.*
import com.claudiator.app.ui.sessions.*
import com.claudiator.app.ui.settings.*
import com.claudiator.app.ui.setup.*
import com.claudiator.app.ui.theme.LocalAppTheme
import kotlinx.coroutines.launch

@Composable
fun AppNavigation(
    app: ClaudiatorApp,
) {
    val isConfigured by app.apiClient.isConfigured.collectAsState()

    val navController = rememberNavController()

    NavHost(
        navController = navController,
        startDestination = if (isConfigured) Screen.Main.route else Screen.Setup.route,
    ) {
        composable(Screen.Setup.route) {
            SetupScreen(
                apiClient = app.apiClient,
                onConnected = {
                    navController.navigate(Screen.Main.route) {
                        popUpTo(Screen.Setup.route) { inclusive = true }
                    }
                },
            )
        }
        composable(Screen.Main.route) {
            MainScaffold(app = app, rootNavController = navController)
        }
        composable(
            Screen.DeviceDetail.route,
            arguments = listOf(navArgument("deviceId") { defaultValue = "" }),
        ) { backStackEntry ->
            val deviceId = backStackEntry.arguments?.getString("deviceId") ?: ""
            DeviceDetailScreen(
                deviceId = deviceId,
                apiClient = app.apiClient,
                versionMonitor = app.versionMonitor,
                notificationManager = app.notificationManager,
                onSessionClick = { sessionId ->
                    navController.navigate(Screen.SessionDetail.createRoute(sessionId))
                },
                onBack = { navController.popBackStack() },
            )
        }
        composable(
            Screen.SessionDetail.route,
            arguments = listOf(navArgument("sessionId") { defaultValue = "" }),
        ) { backStackEntry ->
            val sessionId = backStackEntry.arguments?.getString("sessionId") ?: ""
            SessionDetailScreen(
                sessionId = sessionId,
                apiClient = app.apiClient,
                versionMonitor = app.versionMonitor,
                notificationManager = app.notificationManager,
                onDeviceClick = { deviceId ->
                    navController.navigate(Screen.DeviceDetail.createRoute(deviceId))
                },
                onBack = { navController.popBackStack() },
            )
        }
    }
}

@Composable
fun MainScaffold(
    app: ClaudiatorApp,
    rootNavController: NavHostController,
) {
    val theme = LocalAppTheme.current
    val pagerState = rememberPagerState(initialPage = 1) { 3 }
    val scope = rememberCoroutineScope()

    // Start polling
    LaunchedEffect(Unit) {
        app.versionMonitor.start(this)
    }

    Scaffold(
        bottomBar = {
            NavigationBar {
                val tabs = listOf(
                    Triple(0, "Devices", Icons.Outlined.Devices),
                    Triple(1, "Sessions", Icons.Outlined.Terminal),
                    Triple(2, "Settings", Icons.Outlined.Settings),
                )
                tabs.forEach { (index, label, icon) ->
                    NavigationBarItem(
                        icon = { Icon(icon, contentDescription = label) },
                        label = { Text(label) },
                        selected = pagerState.currentPage == index,
                        onClick = { scope.launch { pagerState.animateScrollToPage(index) } },
                    )
                }
            }
        },
    ) { padding ->
        HorizontalPager(
            state = pagerState,
            modifier = Modifier
                .fillMaxSize()
                .padding(padding),
        ) { page ->
            when (page) {
                0 -> DeviceListScreen(
                    apiClient = app.apiClient,
                    versionMonitor = app.versionMonitor,
                    notificationManager = app.notificationManager,
                    onDeviceClick = { deviceId ->
                        rootNavController.navigate(Screen.DeviceDetail.createRoute(deviceId))
                    },
                )
                1 -> AllSessionsScreen(
                    apiClient = app.apiClient,
                    versionMonitor = app.versionMonitor,
                    notificationManager = app.notificationManager,
                    onSessionClick = { sessionId ->
                        rootNavController.navigate(Screen.SessionDetail.createRoute(sessionId))
                    },
                    onDeviceClick = { deviceId ->
                        rootNavController.navigate(Screen.DeviceDetail.createRoute(deviceId))
                    },
                )
                2 -> SettingsScreen(
                    apiClient = app.apiClient,
                    themeManager = app.themeManager,
                    onDisconnect = {
                        app.apiClient.disconnect()
                        app.versionMonitor.stop()
                        rootNavController.navigate(Screen.Setup.route) {
                            popUpTo(Screen.Main.route) { inclusive = true }
                        }
                    },
                )
            }
        }
    }
}
```

- [ ] **Step 4: Create MainActivity.kt**

```kotlin
// android/app/src/main/java/com/claudiator/app/MainActivity.kt
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

        // Register stored FCM token if not yet registered
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
```

- [ ] **Step 5: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/ClaudiatorApp.kt android/app/src/main/java/com/claudiator/app/MainActivity.kt android/app/src/main/java/com/claudiator/app/navigation/
git commit -m "add app entry point and navigation"
```

---

### Task 10: Shared UI Components

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/ui/components/ThemedCard.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/components/PlatformIcon.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/components/StatusBadge.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/components/ThemedSegmentedPicker.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/components/ThemePreviewCard.kt`

Each component is a direct port of its iOS counterpart using Compose equivalents. Implementation instructions:

- [ ] **Step 1: Create ThemedCard.kt** — Composable that wraps content in `Card` with theme's `cardBackground`, `cardBorder` (with opacity), `cardCornerRadius`, subtle elevation.

- [ ] **Step 2: Create PlatformIcon.kt** — Composable that renders a platform icon. Use Material icons: `Icons.Outlined.Laptop` for mac, `Icons.Outlined.Computer` for linux, `Icons.Outlined.DesktopWindows` for windows, `Icons.Outlined.Monitor` for default. Tinted with `theme.platformColor(platform)`.

- [ ] **Step 3: Create StatusBadge.kt** — Colored pill shape with count + label text. Background is status color at 10% opacity, text in full color.

- [ ] **Step 4: Create ThemedSegmentedPicker.kt** — Port of `ThemedSegmentedPicker`. Row of buttons in capsule shape, selected button has card background + shadow, uses theme colors.

- [ ] **Step 5: Create ThemePreviewCard.kt** — Small card showing 4 color swatches in a row + theme name below. Selected state has highlighted border.

- [ ] **Step 6: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/ui/components/
git commit -m "add shared UI components"
```

---

### Task 11: Setup Screen

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/ui/setup/SetupScreen.kt`

- [ ] **Step 1: Create SetupScreen.kt** — Port of `SetupView.swift`. Centered layout with logo, server URL field (keyboard type URI), API key field (password toggle), error text, Connect button with loading spinner. Uses `SetupViewModel`.

- [ ] **Step 2: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/ui/setup/
git commit -m "add setup screen"
```

---

### Task 12: Device Screens

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/ui/devices/DeviceListScreen.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/devices/DeviceDetailScreen.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/devices/DeviceRow.kt`

- [ ] **Step 1: Create DeviceRow.kt** — Port of `DeviceRow`. Platform icon + device name + "last active" time + status badges row.

- [ ] **Step 2: Create DeviceListScreen.kt** — Port of `DeviceListView`. Top bar with "Devices" title + bell icon with badge. `LazyColumn` of `DeviceRow`. Pull-to-refresh. Empty state. Auto-refresh on `dataVersion` change.

- [ ] **Step 3: Create DeviceDetailScreen.kt** — Port of `DeviceDetailView`. Device header, details section, sessions list with Active/All filter. Pull-to-refresh.

- [ ] **Step 4: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/ui/devices/
git commit -m "add device screens"
```

---

### Task 13: Session Screens

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/ui/sessions/SessionRow.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/sessions/AllSessionRow.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/sessions/DeviceGroupHeader.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/sessions/DeviceGroupCard.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/sessions/EventRow.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/sessions/AllSessionsScreen.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/sessions/SessionDetailScreen.kt`

- [ ] **Step 1: Create SessionRow.kt** — Port of `SessionRow.swift`. Status dot + title/cwd + status label + relative time.

- [ ] **Step 2: Create AllSessionRow.kt** — Port of `AllSessionRow.swift`. Like SessionRow but includes device name + platform icon.

- [ ] **Step 3: Create DeviceGroupHeader.kt** — Port of `DeviceGroupHeader.swift`. Collapsible header with platform icon, device name, session count, expand/collapse chevron.

- [ ] **Step 4: Create DeviceGroupCard.kt** — Port of `DeviceGroupCard.swift`. Card containing header + expandable session list. Used in tablet 2-column grid.

- [ ] **Step 5: Create EventRow.kt** — Port of `EventRow.swift`. Event icon (colored by type) + hook event name + optional tool/notification badges + message + timestamp.

- [ ] **Step 6: Create AllSessionsScreen.kt** — Port of `AllSessionsView.swift`. Three layout modes (flat, grouped compact, grouped wide for tablet via `WindowSizeClass`). Filter toolbar, grouping toggle, pagination sentinel, pull-to-refresh, bell icon, notification pulse animation.

- [ ] **Step 7: Create SessionDetailScreen.kt** — Port of `SessionDetailView.swift`. Status header with colored circle + halo, collapsible details section (device card, cwd, timestamps, session ID), events list, pull-to-refresh, marks session read on composition.

- [ ] **Step 8: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/ui/sessions/
git commit -m "add session screens"
```

---

### Task 14: Notification Screen

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/ui/notifications/NotificationListSheet.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/notifications/NotificationRow.kt`

- [ ] **Step 1: Create NotificationRow.kt** — Port of notification row from `NotificationListView.swift`. Unread dot + type icon + title/body + timestamp. Tappable.

- [ ] **Step 2: Create NotificationListSheet.kt** — Port of `NotificationListView.swift` as `ModalBottomSheet`. Header with "Notifications" + "Mark All Read". `LazyColumn` of `NotificationRow`. Swipe-to-dismiss marks read. Empty state.

- [ ] **Step 3: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/ui/notifications/
git commit -m "add notification list sheet"
```

---

### Task 15: Settings Screen

**Files:**
- Create: `android/app/src/main/java/com/claudiator/app/ui/settings/SettingsScreen.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/settings/AppearanceSection.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/settings/ServerConfigSection.kt`
- Create: `android/app/src/main/java/com/claudiator/app/ui/settings/DangerZoneSection.kt`

- [ ] **Step 1: Create AppearanceSection.kt** — Port of `AppearanceSection.swift`. Mode picker (System/Light/Dark segmented), theme selector with `ThemePreviewCard` row.

- [ ] **Step 2: Create ServerConfigSection.kt** — Port of `ServerConfigSection.swift`. Editable URL/key fields, connection status dot, test/save button.

- [ ] **Step 3: Create DangerZoneSection.kt** — Port of `DangerZoneSection.swift`. Red disconnect button with `AlertDialog` confirmation.

- [ ] **Step 4: Create SettingsScreen.kt** — Port of `SettingsView.swift`. Logo + version, then the three sections in a scrollable column.

- [ ] **Step 5: Commit**

```bash
git add android/app/src/main/java/com/claudiator/app/ui/settings/
git commit -m "add settings screen"
```

---

### Task 16: App Icon & Resources

**Files:**
- Create: `android/app/src/main/res/drawable/ic_launcher_foreground.xml`
- Create: `android/app/src/main/res/mipmap-*/ic_launcher.webp` (or use adaptive icon XML)

- [ ] **Step 1: Create adaptive icon** — Use the existing `claudiator-icon.png` from the repo root. Generate Android adaptive icon resources using Android Studio's Image Asset tool, or create an `ic_launcher_foreground.xml` vector drawable and `ic_launcher_background.xml`.

- [ ] **Step 2: Commit**

```bash
git add android/app/src/main/res/
git commit -m "add app icon and resources"
```

---

### Task 17: README & Documentation

**Files:**
- Create: `android/README.md`

- [ ] **Step 1: Create README.md** with:
- Prerequisites (Android Studio Ladybug+, JDK 17+)
- Firebase setup instructions (create project, add Android app with package `com.claudiator.app`, download `google-services.json` to `android/app/`)
- Build instructions (`./gradlew assembleDebug`)
- Test instructions (`./gradlew test`)
- Release signing instructions
- Architecture overview (brief)

- [ ] **Step 2: Commit**

```bash
git add android/README.md
git commit -m "add android README"
```

---

### Task 18: Integration Testing & Final Build

- [ ] **Step 1: Run all tests**

```bash
cd android && ./gradlew test
```

Expected: all tests pass.

- [ ] **Step 2: Build debug APK**

```bash
cd android && ./gradlew assembleDebug
```

Expected: BUILD SUCCESSFUL, APK at `app/build/outputs/apk/debug/app-debug.apk`.

- [ ] **Step 3: Manual smoke test** — Install APK on device/emulator, verify:
- Setup screen appears
- Can enter server URL and API key
- Connect succeeds (if test server available)
- Tab navigation works with swipe
- Theme switching works
- Settings persist across restarts

- [ ] **Step 4: Final commit if any fixes were needed**

```bash
git add -A android/
git commit -m "fix integration issues"
```
