import SwiftUI

struct SessionRow: View {
    let session: Session
    @Environment(ThemeManager.self) private var themeManager

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
                Text(statusDisplayLabel(session.status))
                    .font(.caption)
                    .foregroundStyle(themeManager.current.statusColor(for: session.status))
            }

            Spacer()

            Text(relativeTime(session.lastEvent))
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.vertical, 2)
    }
}
