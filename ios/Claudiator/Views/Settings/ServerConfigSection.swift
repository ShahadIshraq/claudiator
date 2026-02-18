import SwiftUI

struct ServerConfigSection: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager
    @State private var serverStatus: ServerStatus = .checking
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
        guard let url = URLValidator.cleanAndValidate(editURL) else {
            serverStatus = .error("Invalid URL")
            return
        }
        editURL = url

        do {
            try apiClient.configure(url: url, apiKey: editAPIKey)
            hasChanges = false
            await checkStatus()
        } catch {
            serverStatus = .error("Failed to save: \(error.localizedDescription)")
        }
    }
}
