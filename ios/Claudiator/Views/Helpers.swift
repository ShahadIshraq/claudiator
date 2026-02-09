import Foundation
import SwiftUI

/// Returns the asset catalog image name for a platform string
func platformImageName(_ platform: String) -> String {
    switch platform.lowercased() {
    case "mac", "macos", "darwin": "AppleLogo"
    case "linux": "LinuxLogo"
    case "windows": "WindowsLogo"
    default: "WindowsLogo"
    }
}

/// A small platform icon view, consistent everywhere
struct PlatformIcon: View {
    @Environment(ThemeManager.self) private var themeManager
    let platform: String
    var size: CGFloat = 18

    private var isTemplate: Bool {
        platform.lowercased() == "mac" || platform.lowercased() == "macos" || platform.lowercased() == "darwin"
    }

    var body: some View {
        Image(platformImageName(platform))
            .resizable()
            .aspectRatio(contentMode: .fit)
            .frame(width: size, height: size)
            .foregroundStyle(isTemplate ? themeManager.current.platformColor(for: platform) : .primary)
    }
}

@MainActor
private enum FormatterCache {
    static let iso8601: ISO8601DateFormatter = {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return f
    }()

    static let iso8601NoFractional: ISO8601DateFormatter = {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime]
        return f
    }()

    static let relative: RelativeDateTimeFormatter = {
        let f = RelativeDateTimeFormatter()
        f.unitsStyle = .abbreviated
        return f
    }()
}

@MainActor
func relativeTime(_ isoString: String) -> String {
    guard let date = FormatterCache.iso8601.date(from: isoString)
        ?? FormatterCache.iso8601NoFractional.date(from: isoString) else {
        return isoString
    }
    return FormatterCache.relative.localizedString(for: date, relativeTo: Date())
}

/// Extracts last 2 path components from a working directory path for display
func cwdShortDisplay(_ cwd: String) -> String {
    let components = cwd.split(separator: "/")
    if components.count >= 2 {
        return components.suffix(2).joined(separator: "/")
    }
    let winComponents = cwd.split(separator: "\\")
    if winComponents.count >= 2 {
        return winComponents.suffix(2).joined(separator: "/")
    }
    return cwd
}

/// Formats a status string for display (e.g. "waiting_for_input" -> "Waiting For Input")
func statusDisplayLabel(_ status: String) -> String {
    switch status {
    case "active": "Active"
    case "waiting_for_input": "Waiting for Input"
    case "waiting_for_permission": "Waiting for Permission"
    case "idle": "Idle"
    case "ended": "Ended"
    default: status.replacingOccurrences(of: "_", with: " ").capitalized
    }
}

/// Calculates the priority status for a collection of sessions
/// Priority order: waiting_for_permission > waiting_for_input > active > idle > ended
func priorityStatus(for sessions: [Session]) -> String {
    if sessions.contains(where: { $0.status == "waiting_for_permission" }) {
        return "waiting_for_permission"
    }
    if sessions.contains(where: { $0.status == "waiting_for_input" }) {
        return "waiting_for_input"
    }
    if sessions.contains(where: { $0.status == "active" }) {
        return "active"
    }
    if sessions.contains(where: { $0.status == "idle" }) {
        return "idle"
    }
    return "ended"
}

// MARK: - Themed Segmented Picker

struct ThemedSegmentedPicker<Option: Hashable & CaseIterable & RawRepresentable>: View where Option.RawValue == String,
    Option.AllCases: RandomAccessCollection {
    @Environment(ThemeManager.self) private var themeManager
    @Binding var selection: Option
    let options: Option.AllCases

    var body: some View {
        HStack(spacing: 0) {
            ForEach(Array(options), id: \.rawValue) { option in
                Button {
                    withAnimation(.easeInOut(duration: 0.2)) {
                        selection = option
                    }
                } label: {
                    Text(option.rawValue)
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 8)
                        .background(
                            Capsule()
                                .fill(selection == option ? themeManager.current.cardBackground : .clear)
                                .shadow(color: selection == option ? .black.opacity(0.06) : .clear, radius: 2, y: 1)
                        )
                }
                .buttonStyle(.plain)
            }
        }
        .padding(3)
        .background(
            Capsule()
                .fill(themeManager.current.cardBackground.opacity(0.5))
                .overlay(
                    Capsule()
                        .strokeBorder(themeManager.current.cardBorder.opacity(AppTheme.cardBorderOpacity), lineWidth: AppTheme.cardBorderWidth)
                )
        )
        .frame(maxWidth: 200)
        .frame(maxWidth: .infinity)
    }
}

// MARK: - Themed Card ViewModifier

struct ThemedCardModifier: ViewModifier {
    @Environment(ThemeManager.self) private var themeManager
    var withSeparator: Bool = false

    func body(content: Content) -> some View {
        content
            .listRowSeparator(withSeparator ? .automatic : .hidden)
            .listRowBackground(
                RoundedRectangle(cornerRadius: AppTheme.cardCornerRadius)
                    .fill(themeManager.current.cardBackground)
                    .overlay(
                        RoundedRectangle(cornerRadius: AppTheme.cardCornerRadius)
                            .strokeBorder(
                                themeManager.current.cardBorder.opacity(AppTheme.cardBorderOpacity),
                                lineWidth: AppTheme.cardBorderWidth
                            )
                    )
                    .padding(.vertical, 2)
            )
    }
}

extension View {
    func themedCard(withSeparator: Bool = false) -> some View {
        modifier(ThemedCardModifier(withSeparator: withSeparator))
    }
}

// MARK: - Themed Page Background

struct ThemedPageModifier: ViewModifier {
    @Environment(ThemeManager.self) private var themeManager

    func body(content: Content) -> some View {
        content
            .scrollContentBackground(.hidden)
            .background(themeManager.current.pageBackground)
    }
}

extension View {
    func themedPage() -> some View {
        modifier(ThemedPageModifier())
    }
}
