import SwiftUI

struct SessionDetailView: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager
    @Environment(VersionMonitor.self) private var versionMonitor
    @Environment(NotificationManager.self) private var notificationManager
    @State private var viewModel = EventListViewModel()
    @State private var device: Device?
    @State private var showDetails = true
    let session: Session

    var body: some View {
        List {
            // Session status header
            Section {
                HStack(spacing: 16) {
                    Circle()
                        .fill(themeManager.current.statusColor(for: session.status))
                        .frame(width: 14, height: 14)
                        .overlay(
                            Circle()
                                .fill(themeManager.current.statusHalo(themeManager.current.statusColor(for: session.status)))
                                .frame(width: 28, height: 28)
                        )
                        .frame(width: 40, height: 40)

                    VStack(alignment: .leading, spacing: 4) {
                        if let title = session.title {
                            Text(title)
                                .font(.title3)
                                .fontWeight(.semibold)
                                .lineLimit(2)
                        }
                        Text(statusLabel)
                            .font(session.title != nil ? .subheadline : .title3)
                            .fontWeight(.semibold)
                            .foregroundStyle(themeManager.current.statusColor(for: session.status))
                        if let cwd = session.cwd {
                            Text(cwd)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .lineLimit(2)
                        }
                    }
                }
                .padding(.vertical, 4)
                .themedCard()
            }

            // Session details (collapsible)
            Section {
                Button {
                    withAnimation { showDetails.toggle() }
                } label: {
                    HStack {
                        Text("Details")
                            .font(.headline)
                            .foregroundStyle(.primary)
                        Spacer()
                        Image(systemName: "chevron.right")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .rotationEffect(.degrees(showDetails ? 90 : 0))
                    }
                }
                .themedCard()

                if showDetails {
                    // Device card at top
                    let devicePlatform = session.platform ?? device?.platform ?? "unknown"
                    let deviceName = session.deviceName ?? device?.deviceName ?? "Unknown"
                    if let device {
                        NavigationLink(value: device) {
                            HStack(spacing: 14) {
                                PlatformIcon(platform: devicePlatform, size: 24)
                                    .frame(width: 40, height: 40)
                                    .background(themeManager.current.iconBackground(themeManager.current.platformColor(for: devicePlatform)))
                                    .clipShape(RoundedRectangle(cornerRadius: 8))

                                VStack(alignment: .leading, spacing: 4) {
                                    Text(deviceName)
                                        .font(.headline)
                                    Text(devicePlatform.capitalized)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                            }
                            .padding(.vertical, 4)
                        }
                        .themedCard()
                    } else {
                        HStack(spacing: 14) {
                            PlatformIcon(platform: devicePlatform, size: 24)
                                .frame(width: 40, height: 40)
                                .background(themeManager.current.iconBackground(themeManager.current.platformColor(for: devicePlatform)))
                                .clipShape(RoundedRectangle(cornerRadius: 8))

                            VStack(alignment: .leading, spacing: 4) {
                                Text(deviceName)
                                    .font(.headline)
                                Text(devicePlatform.capitalized)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                        }
                        .padding(.vertical, 4)
                        .themedCard()
                    }

                    if let cwd = session.cwd {
                        LabeledContent("Working Directory") {
                            Text(cwdShortDisplay(cwd))
                                .lineLimit(1)
                                .truncationMode(.head)
                        }
                        .themedCard()
                        LabeledContent("Full Path") {
                            Text(cwd)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .lineLimit(2)
                                .textSelection(.enabled)
                        }
                        .themedCard()
                    }
                    LabeledContent("Status") {
                        HStack(spacing: 6) {
                            Circle()
                                .fill(themeManager.current.statusColor(for: session.status))
                                .frame(width: 8, height: 8)
                            Text(statusLabel)
                                .foregroundStyle(themeManager.current.statusColor(for: session.status))
                        }
                    }
                    .themedCard()
                    LabeledContent("Started") {
                        Text(relativeTime(session.startedAt))
                    }
                    .themedCard()
                    LabeledContent("Last Event") {
                        Text(relativeTime(session.lastEvent))
                    }
                    .themedCard()
                    LabeledContent("Session ID") {
                        Text(session.sessionId)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                            .truncationMode(.middle)
                            .textSelection(.enabled)
                    }
                    .themedCard()
                }
            }

            // Events section
            Section("Events (\(viewModel.events.count))") {
                if viewModel.isLoading && viewModel.events.isEmpty {
                    ProgressView()
                        .frame(maxWidth: .infinity, alignment: .center)
                } else if viewModel.events.isEmpty {
                    Text("No events recorded")
                        .foregroundStyle(.secondary)
                } else {
                    ForEach(viewModel.events) { event in
                        EventRow(event: event)
                            .themedCard()
                    }
                }
            }
        }
        .themedPage()
        .navigationTitle(titleText)
        .navigationBarTitleDisplayMode(.inline)
        .refreshable {
            await viewModel.refresh(apiClient: apiClient, sessionId: session.sessionId)
        }
        .task {
            notificationManager.markSessionRead(session.sessionId)
            if device == nil {
                if let devices = try? await apiClient.fetchDevices() {
                    device = devices.first { $0.deviceId == session.deviceId }
                }
            }
            await viewModel.refresh(apiClient: apiClient, sessionId: session.sessionId)
        }
        .onChange(of: versionMonitor.dataVersion) { _, _ in
            Task { await viewModel.refresh(apiClient: apiClient, sessionId: session.sessionId) }
        }
    }

    private var titleText: String {
        if let title = session.title {
            return title
        }
        if let cwd = session.cwd {
            return cwdShortDisplay(cwd)
        }
        return "Session"
    }

    private var statusLabel: String {
        statusDisplayLabel(session.status)
    }
}
