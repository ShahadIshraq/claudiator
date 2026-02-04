import SwiftUI

struct SettingsView: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager
    @State private var serverStatus: ServerStatus = .checking
    @State private var showDisconnectConfirm = false

    enum ServerStatus {
        case checking, connected, disconnected
    }

    var body: some View {
        List {
            Section {
                HStack(spacing: 14) {
                    Image("ClaudiatorLogo")
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(width: 52, height: 52)
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Claudiator")
                            .font(.title3)
                            .fontWeight(.bold)
                        Text("v1.0.0")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                .padding(.vertical, 4)
                .listRowBackground(Color.clear)
            }

            Section("Appearance") {
                Picker("Mode", selection: Bindable(themeManager).appearance) {
                    ForEach(AppearanceMode.allCases, id: \.self) { mode in
                        Text(mode.rawValue).tag(mode)
                    }
                }
                .pickerStyle(.segmented)
                .listRowBackground(Color.clear)
            }

            Section {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 12) {
                        ForEach(AppTheme.allThemes) { theme in
                            ThemePreviewCard(
                                theme: theme,
                                isSelected: themeManager.current.id == theme.id
                            ) {
                                withAnimation(.easeInOut(duration: 0.2)) {
                                    themeManager.select(theme)
                                }
                            }
                        }
                    }
                    .padding(.horizontal, 4)
                    .padding(.vertical, 8)
                }
                .listRowInsets(EdgeInsets(top: 0, leading: 12, bottom: 0, trailing: 12))
                .themedCard()
            } header: {
                Text("Theme")
            }

            Section("Server Connection") {
                LabeledContent("URL") {
                    Text(apiClient.baseURL)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                        .textSelection(.enabled)
                }
                .themedCard()
                LabeledContent("API Key") {
                    Text(maskedApiKey)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .themedCard()
                LabeledContent("Status") {
                    HStack(spacing: 6) {
                        switch serverStatus {
                        case .checking:
                            ProgressView()
                                .controlSize(.mini)
                            Text("Checking...")
                                .foregroundStyle(.secondary)
                        case .connected:
                            Circle()
                                .fill(themeManager.current.serverConnected)
                                .frame(width: 8, height: 8)
                            Text("Connected")
                                .foregroundStyle(themeManager.current.serverConnected)
                        case .disconnected:
                            Circle()
                                .fill(themeManager.current.serverDisconnected)
                                .frame(width: 8, height: 8)
                            Text("Unreachable")
                                .foregroundStyle(themeManager.current.serverDisconnected)
                        }
                    }
                }
                .themedCard()
            }

            Section {
                Button(role: .destructive) {
                    showDisconnectConfirm = true
                } label: {
                    HStack {
                        Spacer()
                        Text("Disconnect")
                        Spacer()
                    }
                }
                .themedCard()
                .confirmationDialog("Disconnect from server?", isPresented: $showDisconnectConfirm, titleVisibility: .visible) {
                    Button("Disconnect", role: .destructive) {
                        apiClient.disconnect()
                    }
                    Button("Cancel", role: .cancel) {}
                } message: {
                    Text("This will remove your server URL and API key. You'll need to re-enter them to reconnect.")
                }
            }
        }
        .themedPage()
        .navigationTitle("Settings")
        .task {
            await checkStatus()
        }
        .refreshable {
            await checkStatus()
        }
    }

    private var maskedApiKey: String {
        guard let key = try? KeychainService.load(key: "api_key"), !key.isEmpty else {
            return "Not set"
        }
        if key.count <= 6 {
            return String(repeating: "•", count: key.count)
        }
        return String(key.prefix(3)) + String(repeating: "•", count: min(key.count - 6, 10)) + String(key.suffix(3))
    }

    private func checkStatus() async {
        serverStatus = .checking
        do {
            _ = try await apiClient.ping()
            serverStatus = .connected
        } catch {
            serverStatus = .disconnected
        }
    }
}

// MARK: - Theme Preview Card

private struct ThemePreviewCard: View {
    let theme: AppTheme
    let isSelected: Bool
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            VStack(spacing: 8) {
                // Mini mockup
                VStack(spacing: 0) {
                    // Mini nav bar
                    HStack {
                        RoundedRectangle(cornerRadius: 2)
                            .fill(Color.primary.opacity(0.6))
                            .frame(width: 40, height: 6)
                        Spacer()
                    }
                    .padding(.horizontal, 8)
                    .padding(.top, 8)
                    .padding(.bottom, 6)

                    // Mini device rows
                    VStack(spacing: 4) {
                        MiniDeviceRow(
                            platformColor: theme.platformMac,
                            statusColor: theme.statusActive,
                            statusWidth: 22,
                            cardColor: theme.cardBackground
                        )
                        MiniDeviceRow(
                            platformColor: theme.platformLinux,
                            statusColor: theme.statusWaitingInput,
                            statusWidth: 18,
                            cardColor: theme.cardBackground
                        )
                        MiniDeviceRow(
                            platformColor: theme.platformWindows,
                            statusColor: theme.statusIdle,
                            statusWidth: 14,
                            cardColor: theme.cardBackground
                        )
                    }
                    .padding(.horizontal, 6)
                    .padding(.bottom, 8)
                }
                .frame(width: 80, height: 82)
                .background(theme.pageBackground)
                .clipShape(RoundedRectangle(cornerRadius: 10))
                .overlay(
                    RoundedRectangle(cornerRadius: 10)
                        .strokeBorder(
                            isSelected ? theme.statusActive : Color.clear,
                            lineWidth: 2
                        )
                )

                // Label + radio
                Text(theme.name)
                    .font(.caption2)
                    .foregroundStyle(.primary)
                    .lineLimit(1)

                Circle()
                    .strokeBorder(isSelected ? theme.statusActive : Color.gray.opacity(0.4), lineWidth: isSelected ? 0 : 1.5)
                    .background(
                        Circle()
                            .fill(isSelected ? theme.statusActive : Color.clear)
                    )
                    .overlay(
                        isSelected ?
                            Image(systemName: "checkmark")
                                .font(.system(size: 8, weight: .bold))
                                .foregroundStyle(.white)
                            : nil
                    )
                    .frame(width: 18, height: 18)
            }
        }
        .buttonStyle(.plain)
    }
}

private struct MiniDeviceRow: View {
    let platformColor: Color
    let statusColor: Color
    let statusWidth: CGFloat
    let cardColor: Color

    var body: some View {
        HStack(spacing: 4) {
            RoundedRectangle(cornerRadius: 3)
                .fill(platformColor.opacity(0.15))
                .overlay(
                    RoundedRectangle(cornerRadius: 3)
                        .fill(platformColor)
                        .frame(width: 6, height: 6)
                )
                .frame(width: 16, height: 16)

            VStack(alignment: .leading, spacing: 2) {
                RoundedRectangle(cornerRadius: 1)
                    .fill(Color.primary.opacity(0.4))
                    .frame(width: 30, height: 4)
                HStack(spacing: 3) {
                    Circle()
                        .fill(statusColor)
                        .frame(width: 4, height: 4)
                    RoundedRectangle(cornerRadius: 1)
                        .fill(statusColor.opacity(0.4))
                        .frame(width: statusWidth, height: 3)
                }
            }
            Spacer()
        }
        .padding(4)
        .background(cardColor)
        .clipShape(RoundedRectangle(cornerRadius: 5))
    }
}
