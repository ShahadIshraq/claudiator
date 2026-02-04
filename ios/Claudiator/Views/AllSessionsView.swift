import SwiftUI

struct AllSessionsView: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager
    @Environment(VersionMonitor.self) private var versionMonitor
    @State private var viewModel = AllSessionsViewModel()

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
            } else {
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
