import SwiftUI

struct DeviceDetailView: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager
    @Environment(VersionMonitor.self) private var versionMonitor
    @State private var viewModel = SessionListViewModel()
    let device: Device

    var body: some View {
        List {
            // Device info header section
            Section {
                HStack(spacing: 16) {
                    PlatformIcon(platform: device.platform, size: 32)
                        .frame(width: 56, height: 56)
                        .background(themeManager.current.iconBackground(themeManager.current.platformColor(for: device.platform)))
                        .clipShape(RoundedRectangle(cornerRadius: 12))

                    VStack(alignment: .leading, spacing: 4) {
                        Text(device.deviceName)
                            .font(.title3)
                            .fontWeight(.semibold)
                        Text(device.platform.capitalized)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }
                }
                .padding(.vertical, 4)
                .themedCard()
            }

            // Device details section
            Section("Details") {
                LabeledContent("Device ID") {
                    Text(device.deviceId)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
                .themedCard()
                LabeledContent("First Seen") {
                    Text(relativeTime(device.firstSeen))
                }
                .themedCard()
                LabeledContent("Last Active") {
                    Text(relativeTime(device.lastSeen))
                }
                .themedCard()
                LabeledContent("Active Sessions") {
                    Text("\(device.activeSessions)")
                        .fontWeight(.medium)
                        .foregroundStyle(device.activeSessions > 0 ? themeManager.current.uiAccent : .secondary)
                }
                .themedCard()
            }

            // Sessions section
            Section {
                ThemedSegmentedPicker(
                    selection: $viewModel.filter,
                    options: SessionListViewModel.SessionFilter.allCases
                )
                .listRowBackground(Color.clear)
                .listRowSeparator(.hidden)

                if viewModel.isLoading, viewModel.sessions.isEmpty {
                    ProgressView()
                        .frame(maxWidth: .infinity, alignment: .center)
                } else if viewModel.sessions.isEmpty {
                    Text("No sessions found")
                        .foregroundStyle(.secondary)
                } else {
                    ForEach(viewModel.sessions) { session in
                        NavigationLink(value: session) {
                            SessionRow(session: session)
                        }
                        .themedCard()
                    }
                }
            }
        }
        .themedPage()
        .navigationTitle(device.deviceName)
        .navigationBarTitleDisplayMode(.inline)
        .navigationDestination(for: Session.self) { session in
            SessionDetailView(session: session)
        }
        .refreshable {
            await viewModel.refresh(apiClient: apiClient, deviceId: device.deviceId)
        }
        .task(id: viewModel.filter) {
            await viewModel.refresh(apiClient: apiClient, deviceId: device.deviceId)
        }
        .onChange(of: versionMonitor.dataVersion) { _, _ in
            Task { await viewModel.refresh(apiClient: apiClient, deviceId: device.deviceId) }
        }
    }
}
