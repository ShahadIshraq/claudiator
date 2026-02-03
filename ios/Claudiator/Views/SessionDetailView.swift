import SwiftUI

struct SessionDetailView: View {
    @Environment(APIClient.self) private var apiClient
    @Environment(ThemeManager.self) private var themeManager
    @State private var viewModel = EventListViewModel()
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
                        Text(statusLabel)
                            .font(.title3)
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

            // Session details
            Section("Details") {
                if let title = session.title {
                    LabeledContent("Title") {
                        Text(title)
                            .lineLimit(3)
                    }
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
            while !Task.isCancelled {
                await viewModel.refresh(apiClient: apiClient, sessionId: session.sessionId)
                try? await Task.sleep(for: .seconds(10))
            }
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
