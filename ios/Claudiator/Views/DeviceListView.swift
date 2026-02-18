import SwiftUI

struct DeviceListView: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager
    @Environment(VersionMonitor.self) private var versionMonitor
    @Environment(NotificationManager.self) private var notificationManager
    @State private var viewModel = DeviceListViewModel()
    @State private var showNotifications = false

    var body: some View {
        Group {
            if viewModel.isLoading {
                ProgressView("Loading devices...")
            } else if viewModel.devices.isEmpty {
                ContentUnavailableView(
                    "No Devices",
                    systemImage: "desktopcomputer",
                    description: Text("No Claude Code devices have connected to the server yet.")
                )
            } else {
                List(viewModel.devices) { device in
                    NavigationLink(value: device) {
                        DeviceRow(
                            device: device,
                            counts: viewModel.statusCounts[device.deviceId] ?? SessionStatusCounts()
                        )
                    }
                    .themedCard()
                }
                .scrollContentBackground(.hidden)
                .refreshable {
                    await viewModel.refresh(apiClient: apiClient)
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(themeManager.current.pageBackground)
        .navigationTitle("Devices")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .navigationBarLeading) {
                Button {
                    showNotifications = true
                } label: {
                    Image(systemName: "bell")
                        .overlay(alignment: .topTrailing) {
                            if notificationManager.unreadCount > 0 {
                                Text("\(notificationManager.unreadCount)")
                                    .font(.system(size: 10, weight: .bold))
                                    .foregroundStyle(.white)
                                    .padding(3)
                                    .background(themeManager.current.uiError)
                                    .clipShape(Circle())
                                    .offset(x: 6, y: -6)
                            }
                        }
                }
            }
            ToolbarItem(placement: .principal) {
                Text("")
            }
        }
        .safeAreaInset(edge: .top) {
            Text("Devices")
                .font(.largeTitle)
                .fontWeight(.bold)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.leading, 20)
                .padding(.top, 8)
                .padding(.bottom, 0)
                .background(themeManager.current.pageBackground)
        }
        .navigationDestination(for: Device.self) { device in
            DeviceDetailView(device: device)
        }
        .sheet(isPresented: $showNotifications) {
            NotificationListView()
        }
        .task {
            viewModel.apiClient = apiClient
            await viewModel.refresh(apiClient: apiClient)
        }
        .onChange(of: versionMonitor.dataVersion) { _, _ in
            Task { await viewModel.refresh(apiClient: apiClient) }
        }
        .overlay(alignment: .top) {
            if let error = viewModel.errorMessage, !viewModel.devices.isEmpty {
                Text(error)
                    .font(.caption)
                    .padding(8)
                    .frame(maxWidth: .infinity)
                    .background(themeManager.current.uiError.opacity(0.1))
                    .foregroundStyle(themeManager.current.uiError)
            }
        }
    }
}

struct DeviceRow: View {
    @Environment(ThemeManager.self) private var themeManager
    let device: Device
    let counts: SessionStatusCounts

    var body: some View {
        HStack(spacing: 14) {
            PlatformIcon(platform: device.platform, size: 24)
                .frame(width: 40, height: 40)
                .background(themeManager.current.iconBackground(themeManager.current.platformColor(for: device.platform)))
                .clipShape(RoundedRectangle(cornerRadius: 8))

            VStack(alignment: .leading, spacing: 6) {
                Text(device.deviceName)
                    .font(.headline)
                HStack(spacing: 6) {
                    Text("Last active \(relativeTime(device.lastSeen))")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                if counts.totalActive > 0 {
                    HStack(spacing: 6) {
                        if counts.active > 0 {
                            StatusBadge(count: counts.active, color: themeManager.current.statusActive, label: "active")
                        }
                        if counts.waitingInput > 0 {
                            StatusBadge(count: counts.waitingInput, color: themeManager.current.statusWaitingInput, label: "waiting")
                        }
                        if counts.waitingPermission > 0 {
                            StatusBadge(count: counts.waitingPermission, color: themeManager.current.statusWaitingPermission, label: "permission")
                        }
                        if counts.idle > 0 {
                            StatusBadge(count: counts.idle, color: themeManager.current.statusIdle, label: "idle")
                        }
                    }
                }
            }
        }
        .padding(.vertical, 4)
    }
}

struct StatusBadge: View {
    let count: Int
    let color: Color
    let label: String

    var body: some View {
        HStack(spacing: 4) {
            Circle()
                .fill(color)
                .frame(width: 6, height: 6)
            Text("\(count) \(label)")
                .font(.caption2)
                .foregroundStyle(color)
        }
        .padding(.horizontal, 6)
        .padding(.vertical, 3)
        .background(color.opacity(0.1))
        .clipShape(Capsule())
    }
}
