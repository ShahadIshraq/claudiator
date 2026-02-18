import SwiftUI

struct DeviceGroupCard: View {
    @Environment(ThemeManager.self) private var themeManager
    @Environment(NotificationManager.self) private var notificationManager
    let deviceId: String
    let deviceName: String
    let platform: String
    let sessions: [Session]
    let isExpanded: Bool
    let onToggle: () -> Void
    @Binding var notificationPulse: Bool

    private var priorityStatusValue: String {
        priorityStatus(for: sessions)
    }

    private var hasNotifications: Bool {
        sessions.contains { notificationManager.sessionsWithNotifications.contains($0.sessionId) }
    }

    private func sessionCardBrightness(_ hasNotification: Bool) -> Double {
        hasNotification ? (notificationPulse ? 0.12 : 0.04) : 0
    }

    private func groupContainerOpacity(_ hasNotifications: Bool) -> Double {
        hasNotifications ? (notificationPulse ? 0.7 : 0.5) : 0.3
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Header - plain style like narrow layout
            Button(action: onToggle) {
                HStack(spacing: 12) {
                    Circle()
                        .fill(themeManager.current.statusColor(for: priorityStatusValue))
                        .frame(width: 10, height: 10)

                    PlatformIcon(platform: platform, size: 20)

                    VStack(alignment: .leading, spacing: 2) {
                        Text(deviceName)
                            .font(.headline)
                            .foregroundStyle(.primary)
                        Text("\(sessions.count) session\(sessions.count == 1 ? "" : "s")")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()

                    Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .padding(.vertical, 8)
                .padding(.horizontal, 12)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            // Sessions - individual cards when expanded
            if isExpanded {
                VStack(spacing: 8) {
                    ForEach(sessions) { session in
                        let hasNotification = notificationManager.sessionsWithNotifications.contains(session.sessionId)
                        NavigationLink(value: session) {
                            AllSessionRow(
                                session: session,
                                deviceName: deviceName,
                                platform: platform
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
