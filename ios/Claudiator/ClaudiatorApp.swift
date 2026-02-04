import SwiftUI
import UserNotifications

class AppDelegate: NSObject, UIApplicationDelegate, UNUserNotificationCenterDelegate {
    var apiClient: APIClient?

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        UNUserNotificationCenter.current().delegate = self
        return true
    }

    func application(_ application: UIApplication, didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
        let token = deviceToken.map { String(format: "%02x", $0) }.joined()
        print("[Push] Device token: \(token)")

        guard let apiClient else { return }

        let sandbox = Self.isSandboxEnvironment()

        Task {
            do {
                try await apiClient.registerPushToken(platform: "ios", token: token, sandbox: sandbox)
                print("[Push] Token registered (sandbox: \(sandbox))")
            } catch {
                print("[Push] Failed to register token: \(error)")
            }
        }
    }

    func application(_ application: UIApplication, didFailToRegisterForRemoteNotificationsWithError error: Error) {
        print("[Push] Registration failed: \(error.localizedDescription)")
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

    func userNotificationCenter(_ center: UNUserNotificationCenter,
                                willPresent notification: UNNotification) async -> UNNotificationPresentationOptions {
        return [.banner, .sound]
    }
}

@main
struct ClaudiatorApp: App {
    @UIApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    @State private var apiClient = APIClient()
    @State private var themeManager = ThemeManager()
    @State private var versionMonitor = VersionMonitor()
    @State private var notificationManager = NotificationManager()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(apiClient)
                .environment(themeManager)
                .environment(versionMonitor)
                .environment(notificationManager)
                .preferredColorScheme(themeManager.appearance.colorScheme)
                .onAppear {
                    appDelegate.apiClient = apiClient
                }
        }
    }
}

struct ContentView: View {
    @Environment(APIClient.self) private var apiClient

    var body: some View {
        Group {
            if apiClient.isConfigured {
                MainTabView()
            } else {
                SetupView()
            }
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
