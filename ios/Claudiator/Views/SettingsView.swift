// swiftlint:disable file_length
import SwiftUI

struct SettingsView: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager
    @State private var serverStatus: ServerStatus = .checking
    @State private var showDisconnectConfirm = false
    @State private var editURL: String = ""
    @State private var editAPIKey: String = ""
    @State private var hasChanges = false
    @State private var showAPIKey = false

    enum ServerStatus: Equatable {
        case checking
        case connected
        case unauthorized
        case unreachable
        case error(String)
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
                VStack(alignment: .leading, spacing: 4) {
                    Text("URL")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    TextField("http://localhost:3000", text: $editURL)
                        .font(.body)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .keyboardType(.URL)
                        .onChange(of: editURL) { _, _ in updateHasChanges() }
                }
                .themedCard()
                VStack(alignment: .leading, spacing: 4) {
                    Text("API Key")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    HStack {
                        if showAPIKey {
                            TextField("Enter API key", text: $editAPIKey)
                                .font(.body)
                                .textInputAutocapitalization(.never)
                                .autocorrectionDisabled()
                        } else {
                            SecureField("Enter API key", text: $editAPIKey)
                                .font(.body)
                                .textInputAutocapitalization(.never)
                                .autocorrectionDisabled()
                        }
                        Button {
                            showAPIKey.toggle()
                        } label: {
                            Image(systemName: showAPIKey ? "eye.slash" : "eye")
                                .foregroundStyle(.secondary)
                        }
                        .buttonStyle(.plain)
                    }
                    .onChange(of: editAPIKey) { _, _ in updateHasChanges() }
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
                        case .unauthorized:
                            Circle()
                                .fill(themeManager.current.uiError)
                                .frame(width: 8, height: 8)
                            Text("Invalid API Key")
                                .foregroundStyle(themeManager.current.uiError)
                        case .unreachable:
                            Circle()
                                .fill(themeManager.current.serverDisconnected)
                                .frame(width: 8, height: 8)
                            Text("Unreachable")
                                .foregroundStyle(themeManager.current.serverDisconnected)
                        case let .error(message):
                            Circle()
                                .fill(themeManager.current.uiError)
                                .frame(width: 8, height: 8)
                            Text(message)
                                .foregroundStyle(themeManager.current.uiError)
                                .lineLimit(1)
                        }
                    }
                }
                .themedCard()
                Button {
                    Task { await checkStatus() }
                } label: {
                    HStack {
                        Spacer()
                        if serverStatus == .checking {
                            ProgressView()
                                .controlSize(.small)
                                .padding(.trailing, 4)
                        }
                        Text("Test Connection")
                        Spacer()
                    }
                }
                .disabled(serverStatus == .checking)
                .themedCard()
                if hasChanges {
                    Button {
                        Task { await saveConfig() }
                    } label: {
                        HStack {
                            Spacer()
                            Text("Save")
                                .fontWeight(.semibold)
                            Spacer()
                        }
                    }
                    .themedCard()
                }
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
        .scrollDismissesKeyboard(.interactively)
        .toolbar {
            ToolbarItemGroup(placement: .keyboard) {
                Spacer()
                Button("Done") {
                    UIApplication.shared.sendAction(
                        #selector(UIResponder.resignFirstResponder),
                        to: nil,
                        from: nil,
                        for: nil
                    )
                }
            }
        }
        .navigationTitle("Settings")
        .task {
            editURL = apiClient.baseURL
            editAPIKey = (try? KeychainService.load(key: "api_key")) ?? ""
            await checkStatus()
        }
        .refreshable {
            await checkStatus()
        }
    }

    // swiftlint:disable:next cyclomatic_complexity
    private func checkStatus() async {
        serverStatus = .checking

        // If there are unsaved edits, temporarily swap in the edited values to test them
        let savedURL = apiClient.baseURL
        let savedKey = (try? KeychainService.load(key: "api_key")) ?? ""
        let testingEdits = editURL != savedURL || editAPIKey != savedKey

        if testingEdits {
            do {
                try apiClient.configure(url: editURL, apiKey: editAPIKey)
            } catch {
                serverStatus = .error("Invalid configuration")
                return
            }
        }

        defer {
            if testingEdits {
                apiClient.baseURL = savedURL
                if !savedKey.isEmpty {
                    try? KeychainService.save(key: "api_key", value: savedKey)
                } else {
                    try? KeychainService.delete(key: "api_key")
                }
            }
        }

        do {
            _ = try await apiClient.ping()
            serverStatus = .connected
        } catch let error as APIError {
            switch error {
            case .unauthorized:
                serverStatus = .unauthorized
            case .notConfigured:
                serverStatus = .error("Not configured")
            case .invalidURL:
                serverStatus = .error("Invalid URL")
            case let .serverError(code):
                serverStatus = .error("Server error (\(code))")
            default:
                serverStatus = .unreachable
            }
        } catch is URLError {
            serverStatus = .unreachable
        } catch {
            serverStatus = .error(error.localizedDescription)
        }
    }

    private func updateHasChanges() {
        let currentKey = (try? KeychainService.load(key: "api_key")) ?? ""
        hasChanges = editURL != apiClient.baseURL || editAPIKey != currentKey
    }

    private func saveConfig() async {
        var url = editURL.trimmingCharacters(in: .whitespacesAndNewlines)
        if url.hasSuffix("/") { url.removeLast() }
        if !url.hasPrefix("http") {
            let isLocal = url.contains(".local") || url.starts(with: "localhost") || url.starts(with: "127.0.0.1")
            url = (isLocal ? "http://" : "https://") + url
            editURL = url
        }

        do {
            try apiClient.configure(url: url, apiKey: editAPIKey)
            hasChanges = false
            await checkStatus()
        } catch {
            serverStatus = .error("Failed to save: \(error.localizedDescription)")
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
