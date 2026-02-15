import SwiftUI
import UIKit
import UserNotifications

@MainActor
class AppDelegate: NSObject, UIApplicationDelegate, UNUserNotificationCenterDelegate {
    var apiClient: APIClient?
    var notificationManager: NotificationManager?

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        UNUserNotificationCenter.current().delegate = self
        return true
    }

    /// Handle APNs push notifications (for deduplication)
    func application(
        _ application: UIApplication,
        didReceiveRemoteNotification userInfo: [AnyHashable: Any]
    ) async -> UIBackgroundFetchResult {
        // Extract notification_id from push payload and mark as received
        if let notificationId = userInfo["notification_id"] as? String {
            notificationManager?.markReceivedViaPush(notificationId: notificationId)
        }

        // Trigger immediate poll to update UI (bell icon, borders, notification list)
        if let apiClient, let notificationManager {
            await notificationManager.fetchNewNotifications(apiClient: apiClient)
        }

        return .newData
    }

    func application(_ application: UIApplication, didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
        let token = deviceToken.map { String(format: "%02x", $0) }.joined()
        guard let apiClient else { return }

        let sandbox = Self.isSandboxEnvironment()

        Task {
            do {
                try await apiClient.registerPushToken(platform: "ios", token: token, sandbox: sandbox)
            } catch {
                // Silent failure - push token registration is non-critical
            }
        }
    }

    func application(_ application: UIApplication, didFailToRegisterForRemoteNotificationsWithError error: Error) {
        // Silent failure - push token registration is non-critical
    }

    /// Show notification banners even when app is in foreground
    nonisolated func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification
    ) async -> UNNotificationPresentationOptions {
        [.banner, .sound, .badge]
    }

    private static func isSandboxEnvironment() -> Bool {
        #if DEBUG
            return true
        #else
            // Check embedded provisioning profile for aps-environment
            guard let url = Bundle.main.url(forResource: "embedded", withExtension: "mobileprovision"),
                  let data = try? Data(contentsOf: url),
                  let content = String(data: data, encoding: .ascii) else {
                return false
            }
            return !content.contains("<string>production</string>")
        #endif
    }
}

@main
struct ClaudiatorApp: App {
    @UIApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    @State private var apiClient = APIClient()
    @State private var themeManager = ThemeManager()
    @State private var versionMonitor = VersionMonitor()
    @State private var notificationManager = NotificationManager()
    @State private var setupViewModel = SetupViewModel()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(apiClient)
                .environment(themeManager)
                .environment(versionMonitor)
                .environment(notificationManager)
                .environment(setupViewModel)
                .preferredColorScheme(themeManager.appearance.colorScheme)
                .onAppear {
                    appDelegate.apiClient = apiClient
                    appDelegate.notificationManager = notificationManager
                }
        }
    }
}

struct ContentView: View {
    @Environment(APIClient.self) private var apiClient
    @State private var refreshID = UUID()

    var body: some View {
        Group {
            if apiClient.isConfigured {
                MainTabView()
            } else {
                SetupView()
            }
        }
        .id(refreshID)
        .onReceive(NotificationCenter.default.publisher(for: Notification.Name("RefreshContentView"))) { _ in
            refreshID = UUID()
        }
    }
}

struct MainTabView: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager
    @Environment(VersionMonitor.self) private var versionMonitor
    @Environment(NotificationManager.self) private var notificationManager
    @State private var selectedTab = 1
    @State private var devicesPath = NavigationPath()
    @State private var sessionsPath = NavigationPath()
    @State private var settingsPath = NavigationPath()

    @GestureState private var dragOffset: CGFloat = 0

    private var isOnDetailView: Bool {
        !devicesPath.isEmpty || !sessionsPath.isEmpty || !settingsPath.isEmpty
    }

    var body: some View {
        VStack(spacing: 0) {
            GeometryReader { geometry in
                HStack(spacing: 0) {
                    NavigationStack(path: $devicesPath) {
                        DeviceListView()
                    }
                    .frame(width: geometry.size.width)

                    NavigationStack(path: $sessionsPath) {
                        AllSessionsView()
                    }
                    .frame(width: geometry.size.width)

                    NavigationStack(path: $settingsPath) {
                        SettingsView()
                    }
                    .frame(width: geometry.size.width)
                }
                .frame(width: geometry.size.width, alignment: .leading)
                .offset(x: -CGFloat(selectedTab) * geometry.size.width)
                .offset(x: isOnDetailView ? 0 : dragOffset)
                .animation(.interactiveSpring(), value: selectedTab)
                .animation(.interactiveSpring(), value: dragOffset)
                .gesture(
                    isOnDetailView ? nil :
                        DragGesture(minimumDistance: 20)
                        .updating($dragOffset) { value, state, _ in
                            let horizontal = value.translation.width
                            let vertical = value.translation.height
                            if abs(horizontal) > abs(vertical) {
                                state = horizontal
                            }
                        }
                        .onEnded { value in
                            let horizontal = value.translation.width
                            let vertical = value.translation.height
                            guard abs(horizontal) > abs(vertical) else { return }
                            let threshold = geometry.size.width * 0.25
                            let velocity = value.predictedEndTranslation.width
                            if horizontal < -threshold || velocity < -500 {
                                selectedTab = min(selectedTab + 1, 2)
                            } else if horizontal > threshold || velocity > 500 {
                                selectedTab = max(selectedTab - 1, 0)
                            }
                        }
                )
            }

            if !isOnDetailView {
                // Custom Tab Bar
                VStack(spacing: 0) {
                    Divider()

                    HStack(spacing: 0) {
                        TabBarItem(
                            icon: "desktopcomputer",
                            title: "Devices",
                            isSelected: selectedTab == 0,
                            tintColor: themeManager.current.uiTint
                        ) {
                            selectedTab = 0
                        }

                        TabBarItem(
                            icon: "terminal",
                            title: "Sessions",
                            isSelected: selectedTab == 1,
                            tintColor: themeManager.current.uiTint
                        ) {
                            selectedTab = 1
                        }

                        TabBarItem(
                            icon: "gearshape",
                            title: "Settings",
                            isSelected: selectedTab == 2,
                            tintColor: themeManager.current.uiTint
                        ) {
                            selectedTab = 2
                        }
                    }
                    .padding(.top, 8)
                    .padding(.bottom, 4)
                }
                .background(.ultraThinMaterial)
                .transition(.move(edge: .bottom).combined(with: .opacity))
            }
        }
        .animation(.easeInOut(duration: 0.2), value: isOnDetailView)
        .ignoresSafeArea(.keyboard)
        .onAppear {
            versionMonitor.start(apiClient: apiClient, notificationManager: notificationManager)
            Task {
                _ = await NotificationService.requestPermissionAndRegister()
            }
        }
    }
}

struct TabBarItem: View {
    let icon: String
    let title: String
    let isSelected: Bool
    let tintColor: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 4) {
                Image(systemName: icon)
                    .font(.system(size: 24))
                Text(title)
                    .font(.caption2)
            }
            .foregroundStyle(isSelected ? tintColor : .secondary)
            .frame(maxWidth: .infinity)
        }
    }
}
