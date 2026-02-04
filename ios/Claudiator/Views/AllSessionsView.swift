import SwiftUI

struct AllSessionsView: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager
    @Environment(VersionMonitor.self) private var versionMonitor
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass
    @State private var viewModel = AllSessionsViewModel()

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

    private func priorityStatus(for deviceId: String) -> String {
        guard let sessions = viewModel.groupedSessions[deviceId] else { return "ended" }

        // Priority order: waiting_for_permission > waiting_for_input > active > idle > ended
        if sessions.contains(where: { $0.status == "waiting_for_permission" }) {
            return "waiting_for_permission"
        }
        if sessions.contains(where: { $0.status == "waiting_for_input" }) {
            return "waiting_for_input"
        }
        if sessions.contains(where: { $0.status == "active" }) {
            return "active"
        }
        if sessions.contains(where: { $0.status == "idle" }) {
            return "idle"
        }
        return "ended"
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
                    NavigationLink(value: session) {
                        AllSessionRow(session: session, deviceName: session.deviceName ?? "Unknown", platform: session.platform ?? "unknown")
                    }
                    .themedCard()
                }
                .scrollContentBackground(.hidden)
                .refreshable {
                    await viewModel.refresh(apiClient: apiClient)
                }
            } else if !useWideLayout {
                // Grouped narrow: List with collapsible sections
                List {
                    ForEach(sortedDeviceIds, id: \.self) { deviceId in
                        Section {
                            if viewModel.expandedDevices.contains(deviceId) {
                                ForEach(viewModel.groupedSessions[deviceId] ?? []) { session in
                                    NavigationLink(value: session) {
                                        AllSessionRow(session: session, deviceName: session.deviceName ?? "Unknown", platform: session.platform ?? "unknown")
                                    }
                                    .themedCard()
                                }
                            }
                        } header: {
                            DeviceGroupHeader(
                                deviceId: deviceId,
                                deviceName: deviceName(for: deviceId),
                                platform: platform(for: deviceId),
                                sessionCount: viewModel.groupedSessions[deviceId]?.count ?? 0,
                                isExpanded: viewModel.expandedDevices.contains(deviceId),
                                status: priorityStatus(for: deviceId),
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
                        }
                    }
                }
                .scrollContentBackground(.hidden)
                .refreshable {
                    await viewModel.refresh(apiClient: apiClient)
                }
            } else {
                // Grouped wide: Grid with device cards
                ScrollView {
                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 16) {
                        ForEach(sortedDeviceIds, id: \.self) { deviceId in
                            DeviceGroupCard(
                                deviceId: deviceId,
                                deviceName: deviceName(for: deviceId),
                                platform: platform(for: deviceId),
                                sessions: viewModel.groupedSessions[deviceId] ?? []
                            )
                        }
                    }
                    .padding()
                }
                .refreshable {
                    await viewModel.refresh(apiClient: apiClient)
                }
            }
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
    let session: Session
    let deviceName: String
    let platform: String

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
    let deviceId: String
    let deviceName: String
    let platform: String
    let sessions: [Session]

    private var priorityStatus: String {
        if sessions.contains(where: { $0.status == "waiting_for_permission" }) {
            return "waiting_for_permission"
        }
        if sessions.contains(where: { $0.status == "waiting_for_input" }) {
            return "waiting_for_input"
        }
        if sessions.contains(where: { $0.status == "active" }) {
            return "active"
        }
        if sessions.contains(where: { $0.status == "idle" }) {
            return "idle"
        }
        return "ended"
    }

    var body: some View {
        NavigationLink(value: Device(
            deviceId: deviceId,
            deviceName: deviceName,
            platform: platform,
            firstSeen: "",
            lastSeen: "",
            activeSessions: sessions.count
        )) {
            VStack(alignment: .leading, spacing: 12) {
                HStack(spacing: 12) {
                    Circle()
                        .fill(themeManager.current.statusColor(for: priorityStatus))
                        .frame(width: 10, height: 10)

                    PlatformIcon(platform: platform, size: 24)

                    VStack(alignment: .leading, spacing: 2) {
                        Text(deviceName)
                            .font(.headline)
                            .foregroundStyle(.primary)
                        Text("\(sessions.count) session\(sessions.count == 1 ? "" : "s")")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()
                }

                Divider()

                VStack(alignment: .leading, spacing: 8) {
                    ForEach(sessions.prefix(3)) { session in
                        HStack(spacing: 8) {
                            Circle()
                                .fill(themeManager.current.statusColor(for: session.status))
                                .frame(width: 8, height: 8)

                            Text(session.title ?? cwdShortDisplay(session.cwd ?? session.sessionId))
                                .font(.caption)
                                .lineLimit(1)
                                .foregroundStyle(.primary)

                            Spacer()

                            Text(relativeTime(session.lastEvent))
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }

                    if sessions.count > 3 {
                        Text("+\(sessions.count - 3) more")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .padding()
            .background(
                RoundedRectangle(cornerRadius: AppTheme.cardCornerRadius)
                    .fill(themeManager.current.cardBackground)
                    .overlay(
                        RoundedRectangle(cornerRadius: AppTheme.cardCornerRadius)
                            .strokeBorder(
                                themeManager.current.cardBorder.opacity(AppTheme.cardBorderOpacity),
                                lineWidth: AppTheme.cardBorderWidth
                            )
                    )
            )
        }
        .buttonStyle(.plain)
    }
}
