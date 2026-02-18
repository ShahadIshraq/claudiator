import SwiftUI

struct DeviceGroupHeader: View {
    @Environment(ThemeManager.self) private var themeManager
    let deviceId: String
    let deviceName: String
    let platform: String
    let sessionCount: Int
    let isExpanded: Bool
    let status: String
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 12) {
                Circle()
                    .fill(themeManager.current.statusColor(for: status))
                    .frame(width: 10, height: 10)

                PlatformIcon(platform: platform, size: 20)

                VStack(alignment: .leading, spacing: 2) {
                    Text(deviceName)
                        .font(.headline)
                        .foregroundStyle(.primary)
                    Text("\(sessionCount) session\(sessionCount == 1 ? "" : "s")")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Spacer()

                Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .padding(.vertical, 8)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}
