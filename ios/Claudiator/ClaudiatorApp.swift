import SwiftUI

@main
struct ClaudiatorApp: App {
    @State private var apiClient = APIClient()
    @State private var themeManager = ThemeManager()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(apiClient)
                .environment(themeManager)
                .preferredColorScheme(themeManager.appearance.colorScheme)
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
        .environment(apiClient)
    }
}

struct MainTabView: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager

    var body: some View {
        TabView {
            NavigationStack {
                DeviceListView()
            }
            .tabItem {
                Label("Devices", systemImage: "desktopcomputer")
            }

            NavigationStack {
                AllSessionsView()
            }
            .tabItem {
                Label("Sessions", systemImage: "terminal")
            }

            NavigationStack {
                SettingsView()
            }
            .tabItem {
                Label("Settings", systemImage: "gearshape")
            }
        }
        .tint(themeManager.current.uiTint)
    }
}
