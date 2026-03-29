# Claudiator Android App

Android client for [Claudiator](../README.md) — monitor Claude Code sessions across all your devices.

## Prerequisites

- Android Studio Ladybug (2024.2) or later
- JDK 17+
- Android SDK with API 35

## Firebase Setup

Push notifications require Firebase Cloud Messaging:

1. Create a [Firebase project](https://console.firebase.google.com/)
2. Add an Android app with package name `com.claudiator.app`
3. Download `google-services.json` and place it in `android/app/`
4. Uncomment the google-services plugin in `app/build.gradle.kts`
5. Uncomment Firebase dependencies in `app/build.gradle.kts`

Without Firebase, the app works fine using polling-only (10s interval).

## Build

```bash
cd android
./gradlew assembleDebug
```

APK output: `app/build/outputs/apk/debug/app-debug.apk`

## Test

```bash
cd android
./gradlew test
```

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
- **Ktor Client** — HTTP networking
- **EncryptedSharedPreferences** — Secure credential storage
- **Firebase Cloud Messaging** — Push notifications
- **Single Activity** — Compose Navigation with HorizontalPager tabs

## Features

- Device monitoring with session counts and status badges
- Cross-device session list with grouping and pagination
- Session detail with event timeline
- Real-time push notifications (FCM) with polling fallback
- Notification history with mark-read and bulk acknowledge
- 4 themes (Standard, Neon Ops, Solarized, Arctic) + light/dark/system
- Tablet-responsive layouts
- Swipe gesture tab navigation
