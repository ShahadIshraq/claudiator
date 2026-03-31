# Claudiator Android App

Android client for [Claudiator](../README.md) — monitor Claude Code sessions across all your devices.

## Prerequisites

- Android Studio Ladybug (2024.2) or later, or Android SDK command-line tools
- JDK 17+
- Android SDK with API 35 and Build Tools 35.0.0

## Firebase Setup (Push Notifications)

Push notifications require Firebase Cloud Messaging. Without Firebase, the app works fine using polling-only (10s interval).

### Android app setup

1. Create a [Firebase project](https://console.firebase.google.com/)
2. Add an Android app with package name `com.claudiator.app`
3. Download `google-services.json` and place it in `android/app/`

### Server setup (for sending pushes)

1. In Firebase Console → Project Settings → Service accounts → Generate new private key
2. Place the JSON key file on the server
3. Set `CLAUDIATOR_FCM_SERVICE_ACCOUNT=/path/to/key.json` when running the server

## Build

```bash
cd android
./gradlew assembleDebug
```

APK output: `app/build/outputs/apk/debug/app-debug.apk`

## Install & Run (USB device)

```bash
# Install
adb install -r app/build/outputs/apk/debug/app-debug.apk

# Launch
adb shell am start -n com.claudiator.app/.MainActivity

# Force stop
adb shell am force-stop com.claudiator.app

# Build + install + launch (one-liner)
./gradlew assembleDebug && adb install -r app/build/outputs/apk/debug/app-debug.apk && adb shell am force-stop com.claudiator.app && adb shell am start -n com.claudiator.app/.MainActivity

# View crash logs
adb logcat | grep -i "claudiator"
```

## Test

```bash
cd android
./gradlew test
```

73 tests across 7 suites: models, URL validation, helpers, themes, viewmodels, setup, notifications.

## Release Signing

1. Generate a keystore:
   ```bash
   keytool -genkey -v -keystore claudiator.jks -keyalg RSA -keysize 2048 -validity 10000 -alias claudiator
   ```
2. Add signing config to `app/build.gradle.kts`
3. Build release:
   ```bash
   ./gradlew assembleRelease
   ```

## Architecture

- **Kotlin + Jetpack Compose** — Material 3 UI
- **MVVM** — ViewModels with StateFlow
- **Ktor Client** — HTTP networking with retry logic
- **EncryptedSharedPreferences** — Secure credential storage
- **Firebase Cloud Messaging** — Push notifications with polling fallback
- **Single Activity** — Compose Navigation with HorizontalPager tabs

## Project Structure

```
app/src/main/java/com/claudiator/app/
├── ClaudiatorApp.kt              # Application class, service initialization
├── MainActivity.kt               # Single activity entry point
├── navigation/                    # Routes and tab scaffold
├── models/                        # Data models (Device, Session, Event, etc.)
├── services/                      # ApiClient, SecureStorage, NotificationManager, FCM
├── viewmodels/                    # MVVM ViewModels with StateFlow
├── ui/
│   ├── theme/                     # 4 themes, ThemeManager, CompositionLocals
│   ├── components/                # Shared composables (ThemedCard, StatusBadge, etc.)
│   ├── setup/                     # Server connection setup
│   ├── devices/                   # Device list and detail screens
│   ├── sessions/                  # Session list, detail, and event timeline
│   ├── notifications/             # Notification bottom sheet
│   └── settings/                  # Appearance, server config, disconnect
└── util/                          # URLValidator, time formatting, helpers
```
