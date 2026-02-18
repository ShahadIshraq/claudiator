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
                                                AllSessionRow(
                                                    session: session,
                                                    deviceName: session.deviceName ?? "Unknown",
                                                    platform: session.platform ?? "unknown"
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
                Button {
                    viewModel.toggleGrouping()
                } label: {
                    Image(systemName: viewModel.isGroupedByDevice ? "square.grid.2x2.fill" : "square.grid.2x2")
                }
            }
        }
        .task(id: viewModel.filter) {
            viewModel.apiClient = apiClient
            await viewModel.refresh(apiClient: apiClient)
        }
        .onChange(of: versionMonitor.dataVersion) { _, _ in
            Task { await viewModel.refresh(apiClient: apiClient) }
        }
        .task {
            while !Task.isCancelled {
                try? await Task.sleep(for: .seconds(1.2))
                notificationPulse.toggle()
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
