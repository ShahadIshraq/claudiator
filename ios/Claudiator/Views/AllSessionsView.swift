import SwiftUI

struct AllSessionsView: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager
    @Environment(VersionMonitor.self) private var versionMonitor
    @Environment(NotificationManager.self) private var notificationManager
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass
    @State private var viewModel = AllSessionsViewModel()
    @State private var showNotifications = false
    @State private var notificationPulse: Bool = false

    private var useWideLayout: Bool {
        horizontalSizeClass == .regular && viewModel.isGroupedByDevice
    }

    private var sortedDeviceIds: [String] {
        viewModel.groupedSessions.keys.sorted()
    }

    private func deviceName(for deviceId: String) -> String {
        viewModel.groupedSessions[deviceId]?.first?.deviceName ?? "Unknown Device"
    }

    private func platform(for deviceId: String) -> String {
        viewModel.groupedSessions[deviceId]?.first?.platform ?? "unknown"
    }

    private func deviceHasNotifications(deviceId: String) -> Bool {
        guard let sessions = viewModel.groupedSessions[deviceId] else { return false }
        return sessions.contains { notificationManager.sessionsWithNotifications.contains($0.sessionId) }
    }

    private func sessionCardBrightness(_ hasNotification: Bool) -> Double {
        hasNotification ? (notificationPulse ? 0.12 : 0.04) : 0
    }

    private func groupContainerOpacity(_ hasNotifications: Bool) -> Double {
        hasNotifications ? (notificationPulse ? 0.7 : 0.5) : 0.3
    }

    var body: some View {
        Group {
            if viewModel.isLoading {
                ProgressView("Loading sessions...")
            } else if viewModel.sessions.isEmpty {
                ContentUnavailableView(
                    "No Sessions",
                    systemImage: "terminal",
                    description: Text("No active sessions found across any devices.")
                )
            } else if !viewModel.isGroupedByDevice {
                // Ungrouped flat list
                List(viewModel.sessions) { session in
                    let hasNotification = notificationManager.sessionsWithNotifications.contains(session.sessionId)
                    NavigationLink(value: session) {
                        AllSessionRow(session: session, deviceName: session.deviceName ?? "Unknown", platform: session.platform ?? "unknown")
                    }
                    .listRowBackground(
                        RoundedRectangle(cornerRadius: 8)
                            .fill(themeManager.current.cardBackground)
                            .brightness(sessionCardBrightness(hasNotification))
                            .animation(.easeInOut(duration: 1.2), value: notificationPulse)
                    )
                }
                .scrollContentBackground(.hidden)
            } else if !useWideLayout {
                // Grouped narrow: Manual groups with container backgrounds
                ScrollView {
                    LazyVStack(spacing: 12) {
                        ForEach(sortedDeviceIds, id: \.self) { deviceId in
                            let hasNotifications = deviceHasNotifications(deviceId: deviceId)
                            VStack(alignment: .leading, spacing: 0) {
                                // Header
                                DeviceGroupHeader(
                                    deviceId: deviceId,
                                    deviceName: deviceName(for: deviceId),
                                    platform: platform(for: deviceId),
                                    sessionCount: viewModel.groupedSessions[deviceId]?.count ?? 0,
                                    isExpanded: viewModel.expandedDevices.contains(deviceId),
                                    status: priorityStatus(for: viewModel.groupedSessions[deviceId] ?? []),
                                    onTap: {
                                        withAnimation {
                                            if viewModel.expandedDevices.contains(deviceId) {
                                                viewModel.expandedDevices.remove(deviceId)
                                            } else {
                                                viewModel.expandedDevices.insert(deviceId)
                                            }
                                        }
                                    }
                                )
                                .padding(.horizontal, 12)

                                // Sessions
                                if viewModel.expandedDevices.contains(deviceId) {
                                    VStack(spacing: 8) {
                                        ForEach(viewModel.groupedSessions[deviceId] ?? []) { session in
                                            let hasNotification = notificationManager.sessionsWithNotifications.contains(session.sessionId)
                                            NavigationLink(value: session) {
                                                AllSessionRow(session: session, deviceName: session.deviceName ?? "Unknown", platform: session.platform ?? "unknown")
                                            }
                                            .buttonStyle(.plain)
                                            .padding(.horizontal, 12)
                                            .padding(.vertical, 8)
                                            .background(
                                                RoundedRectangle(cornerRadius: AppTheme.cardCornerRadius)
                                                    .fill(themeManager.current.cardBackground)
                                                    .brightness(sessionCardBrightness(hasNotification))
                                                    .animation(.easeInOut(duration: 1.2), value: notificationPulse)
                                                    .overlay(
                                                        RoundedRectangle(cornerRadius: AppTheme.cardCornerRadius)
                                                            .strokeBorder(
                                                                themeManager.current.cardBorder.opacity(AppTheme.cardBorderOpacity),
                                                                lineWidth: AppTheme.cardBorderWidth
                                                            )
                                                    )
                                            )
                                        }
                                    }
                                    .padding(.horizontal, 12)
                                    .padding(.top, 8)
                                    .padding(.bottom, 12)
                                }
                            }
                            .background(
                                RoundedRectangle(cornerRadius: AppTheme.cardCornerRadius)
                                    .fill(themeManager.current.cardBackground.opacity(groupContainerOpacity(hasNotifications)))
                                    .animation(.easeInOut(duration: 1.2), value: notificationPulse)
                            )
                        }
                    }
                    .padding(16)
                }
                .background(themeManager.current.pageBackground)
            } else {
                // Grouped wide: Grid with device cards
                ScrollView {
                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 16) {
                        ForEach(sortedDeviceIds, id: \.self) { deviceId in
                            DeviceGroupCard(
                                deviceId: deviceId,
                                deviceName: deviceName(for: deviceId),
                                platform: platform(for: deviceId),
                                sessions: viewModel.groupedSessions[deviceId] ?? [],
                                isExpanded: viewModel.expandedDevices.contains(deviceId),
                                onToggle: {
                                    withAnimation {
                                        viewModel.toggleDevice(deviceId)
                                    }
                                },
                                notificationPulse: $notificationPulse
                            )
                        }
                    }
                    .padding()
                }
            }
        }
        .refreshable {
            await viewModel.refresh(apiClient: apiClient)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(themeManager.current.pageBackground)
        .navigationTitle("Sessions")
        .navigationBarTitleDisplayMode(.inline)
        .safeAreaInset(edge: .top) {
            Text("Sessions")
                .font(.largeTitle)
                .fontWeight(.bold)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.leading, 20)
                .padding(.top, 8)
                .padding(.bottom, 0)
                .background(themeManager.current.pageBackground)
        }
        .navigationDestination(for: Session.self) { session in
            SessionDetailView(session: session)
        }
        .navigationDestination(for: Device.self) { device in
            DeviceDetailView(device: device)
        }
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
                Picker("Filter", selection: $viewModel.filter) {
                    ForEach(AllSessionsViewModel.SessionFilter.allCases, id: \.self) { filter in
                        Text(filter.rawValue).tag(filter)
                    }
                }
                .pickerStyle(.segmented)
                .frame(width: 150)
            }

            ToolbarItem(placement: .primaryAction) {
                Button(action: {
                    viewModel.toggleGrouping()
                }) {
                    Image(systemName: viewModel.isGroupedByDevice ? "square.grid.2x2.fill" : "square.grid.2x2")
                }
            }
        }
        .task(id: viewModel.filter) {
            await viewModel.refresh(apiClient: apiClient)
        }
        .onChange(of: versionMonitor.dataVersion) { _, _ in
            Task { await viewModel.refresh(apiClient: apiClient) }
        }
        .onAppear {
            Task {
                while !Task.isCancelled {
                    try? await Task.sleep(for: .seconds(1.2))
                    notificationPulse.toggle()
                }
            }
        }
        .sheet(isPresented: $showNotifications) {
            NotificationListView()
        }
        .overlay(alignment: .top) {
            if let error = viewModel.errorMessage, !viewModel.sessions.isEmpty {
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

struct AllSessionRow: View {
    @Environment(ThemeManager.self) private var themeManager
    @Environment(NotificationManager.self) private var notificationManager
    let session: Session
    let deviceName: String
    let platform: String

    private var hasNotification: Bool {
        notificationManager.sessionsWithNotifications.contains(session.sessionId)
    }

    var body: some View {
        HStack(spacing: 12) {
            Circle()
                .fill(themeManager.current.statusColor(for: session.status))
                .frame(width: 10, height: 10)

            VStack(alignment: .leading, spacing: 4) {
                Text(session.title ?? cwdShortDisplay(session.cwd ?? session.sessionId))
                    .font(.subheadline)
                    .fontWeight(.medium)
                    .lineLimit(1)
                HStack(spacing: 4) {
                    PlatformIcon(platform: platform, size: 12)
                    Text(deviceName)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text("·")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text(statusDisplayLabel(session.status))
                        .font(.caption)
                        .foregroundStyle(themeManager.current.statusColor(for: session.status))
                }
            }

            Spacer()

            Text(relativeTime(session.lastEvent))
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.vertical, 2)
    }
}

// MARK: - Device Group Components

struct DeviceGroupHeader: View {
    @Environment(ThemeManager.self) private var themeManager
    let deviceId: String
    let deviceName: String
    let platform: String
    let sessionCount: Int
    let isExpanded: Bool
    let status: String
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 12) {
                Circle()
                    .fill(themeManager.current.statusColor(for: status))
                    .frame(width: 10, height: 10)

                PlatformIcon(platform: platform, size: 20)

                VStack(alignment: .leading, spacing: 2) {
                    Text(deviceName)
                        .font(.headline)
                        .foregroundStyle(.primary)
                    Text("\(sessionCount) session\(sessionCount == 1 ? "" : "s")")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Spacer()

                Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .padding(.vertical, 8)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}

struct DeviceGroupCard: View {
    @Environment(ThemeManager.self) private var themeManager
    @Environment(NotificationManager.self) private var notificationManager
    let deviceId: String
    let deviceName: String
    let platform: String
    let sessions: [Session]
    let isExpanded: Bool
    let onToggle: () -> Void
    @Binding var notificationPulse: Bool

    private var priorityStatusValue: String {
        priorityStatus(for: sessions)
    }

    private var hasNotifications: Bool {
        sessions.contains { notificationManager.sessionsWithNotifications.contains($0.sessionId) }
    }

    private func sessionCardBrightness(_ hasNotification: Bool) -> Double {
        hasNotification ? (notificationPulse ? 0.12 : 0.04) : 0
    }

    private func groupContainerOpacity(_ hasNotifications: Bool) -> Double {
        hasNotifications ? (notificationPulse ? 0.7 : 0.5) : 0.3
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Header - plain style like narrow layout
            Button(action: onToggle) {
                HStack(spacing: 12) {
                    Circle()
                        .fill(themeManager.current.statusColor(for: priorityStatusValue))
                        .frame(width: 10, height: 10)

                    PlatformIcon(platform: platform, size: 20)

                    VStack(alignment: .leading, spacing: 2) {
                        Text(deviceName)
                            .font(.headline)
                            .foregroundStyle(.primary)
                        Text("\(sessions.count) session\(sessions.count == 1 ? "" : "s")")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()

                    Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .padding(.vertical, 8)
                .padding(.horizontal, 12)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            // Sessions - individual cards when expanded
            if isExpanded {
                VStack(spacing: 8) {
                    ForEach(sessions) { session in
                        let hasNotification = notificationManager.sessionsWithNotifications.contains(session.sessionId)
                        NavigationLink(value: session) {
                            AllSessionRow(
                                session: session,
                                deviceName: deviceName,
                                platform: platform
                            )
                        }
                        .buttonStyle(.plain)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 8)
                        .background(
                            RoundedRectangle(cornerRadius: AppTheme.cardCornerRadius)
                                .fill(themeManager.current.cardBackground)
                                .brightness(sessionCardBrightness(hasNotification))
                                .animation(.easeInOut(duration: 1.2), value: notificationPulse)
                                .overlay(
                                    RoundedRectangle(cornerRadius: AppTheme.cardCornerRadius)
                                        .strokeBorder(
                                            themeManager.current.cardBorder.opacity(AppTheme.cardBorderOpacity),
                                            lineWidth: AppTheme.cardBorderWidth
                                        )
                                )
                        )
                    }
                }
                .padding(.horizontal, 12)
                .padding(.bottom, 12)
            }
        }
        .background(
            RoundedRectangle(cornerRadius: AppTheme.cardCornerRadius)
                .fill(themeManager.current.cardBackground.opacity(groupContainerOpacity(hasNotifications)))
                .animation(.easeInOut(duration: 1.2), value: notificationPulse)
        )
    }
}
