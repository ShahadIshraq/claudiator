import SwiftUI

struct NotificationListView: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(APIClient.self) private var apiClient
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
                                            Task {
                                                await notificationManager.markNotificationRead(
                                                    notificationId: notification.notificationId,
                                                    apiClient: apiClient
                                                )
                                            }
                                        } label: {
                                            Label("Mark Read", systemImage: "checkmark")
                                        }
                                        .tint(.blue)
                                    }
                                }
                                .onTapGesture {
                                    Task {
                                        await notificationManager.markNotificationRead(
                                            notificationId: notification.notificationId,
                                            apiClient: apiClient
                                        )
                                    }
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
            "lock.shield"
        case "idle_prompt":
            "moon.zzz"
        case "stop":
            "hand.raised"
        default:
            "bell"
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
