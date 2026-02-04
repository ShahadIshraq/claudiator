import Foundation
import SwiftUI

/// Returns the asset catalog image name for a platform string
func platformImageName(_ platform: String) -> String {
    switch platform.lowercased() {
    case "mac", "macos", "darwin": return "AppleLogo"
    case "linux": return "LinuxLogo"
    case "windows": return "WindowsLogo"
    default: return "WindowsLogo"
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
    case "active": return "Active"
    case "waiting_for_input": return "Waiting for Input"
    case "waiting_for_permission": return "Waiting for Permission"
    case "idle": return "Idle"
    case "ended": return "Ended"
    default: return status.replacingOccurrences(of: "_", with: " ").capitalized
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
