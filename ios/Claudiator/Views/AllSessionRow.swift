import SwiftUI

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
