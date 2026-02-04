import SwiftUI

struct EventRow: View {
    let event: Event
    @Environment(ThemeManager.self) private var themeManager

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: eventIcon)
                .foregroundStyle(themeManager.current.eventColor(for: event.hookEventName))
                .frame(width: 24)

            VStack(alignment: .leading, spacing: 4) {
                Text(event.hookEventName)
                    .font(.subheadline)
                    .fontWeight(.medium)

                if let toolName = event.toolName {
                    Text(toolName)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                if let message = event.message {
                    Text(message)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(2)
                }

                if let notifType = event.notificationType {
                    Text(notifType.replacingOccurrences(of: "_", with: " "))
                        .font(.caption2)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(themeManager.current.notificationBadgeBackground())
                        .foregroundStyle(themeManager.current.eventNotification)
                        .clipShape(Capsule())
                }
            }

            Spacer()

            Text(relativeTime(event.timestamp))
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
        .padding(.vertical, 2)
    }

    private var eventIcon: String {
        switch event.hookEventName {
        case "SessionStart": return "play.circle"
        case "SessionEnd": return "stop.circle"
        case "Stop": return "pause.circle"
        case "Notification": return "bell"
        case "UserPromptSubmit": return "arrow.up.circle"
        default: return "circle"
        }
    }

}
