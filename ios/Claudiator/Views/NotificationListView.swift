import SwiftUI

struct NotificationListView: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(NotificationManager.self) private var notificationManager
    @Environment(ThemeManager.self) private var themeManager

    var body: some View {
        NavigationStack {
            Group {
                if notificationManager.allNotifications.isEmpty {
                    ContentUnavailableView(
                        "No Notifications",
                        systemImage: "bell.slash",
                        description: Text("You don't have any notifications yet")
                    )
                } else {
                    List {
                        ForEach(notificationManager.allNotifications) { notification in
                            NotificationRow(notification: notification)
                                .themedCard()
                                .swipeActions(edge: .trailing, allowsFullSwipe: true) {
                                    if notificationManager.unreadNotifications.contains(where: { $0.notificationId == notification.notificationId }) {
                                        Button {
                                            notificationManager.markNotificationRead(notificationId: notification.notificationId)
                                        } label: {
                                            Label("Mark Read", systemImage: "checkmark")
                                        }
                                        .tint(.blue)
                                    }
                                }
                                .onTapGesture {
                                    notificationManager.markNotificationRead(notificationId: notification.notificationId)
                                }
                        }
                    }
                    .themedPage()
                    .listStyle(.plain)
                }
            }
            .navigationTitle("Notifications")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }
}

// MARK: - NotificationRow

struct NotificationRow: View {
    @Environment(NotificationManager.self) private var notificationManager
    @Environment(ThemeManager.self) private var themeManager

    let notification: AppNotification

    private var isUnread: Bool {
        notificationManager.unreadNotifications.contains(where: { $0.notificationId == notification.notificationId })
    }

    private var iconName: String {
        switch notification.notificationType {
        case "permission_prompt":
            return "lock.shield"
        case "idle_prompt":
            return "moon.zzz"
        case "stop":
            return "hand.raised"
        default:
            return "bell"
        }
    }

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            // Unread dot indicator
            Circle()
                .fill(isUnread ? themeManager.current.uiTint : Color.clear)
                .frame(width: 8, height: 8)
                .padding(.top, 6)

            // Icon
            Image(systemName: iconName)
                .font(.title3)
                .foregroundStyle(themeManager.current.uiTint)
                .frame(width: 24)

            // Content
            VStack(alignment: .leading, spacing: 4) {
                Text(notification.title)
                    .font(.subheadline)
                    .fontWeight(.medium)

                Text(notification.body)
                    .font(.caption)
                    .foregroundStyle(.secondary)

                Text(relativeTime(notification.createdAt))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            Spacer()
        }
        .padding(.vertical, 8)
    }
}
