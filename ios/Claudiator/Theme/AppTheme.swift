import SwiftUI

struct AppTheme: Identifiable, Equatable {
    let id: String
    let name: String
    let preview: [Color] // 3-4 colors for swatch preview

    // Status colors
    let statusActive: Color
    let statusWaitingInput: Color
    let statusWaitingPermission: Color
    let statusIdle: Color
    let statusEnded: Color

    // Platform colors
    let platformMac: Color
    let platformLinux: Color
    let platformWindows: Color
    let platformDefault: Color

    // Event colors
    let eventSessionStart: Color
    let eventSessionEnd: Color
    let eventStop: Color
    let eventNotification: Color
    let eventUserPromptSubmit: Color
    let eventDefault: Color

    // Server colors
    let serverConnected: Color
    let serverDisconnected: Color

    // UI colors
    let uiError: Color
    let uiAccent: Color
    let uiTint: Color

    // Surface colors (adaptive light/dark)
    let pageBackground: Color
    let cardBackground: Color
    let cardBorder: Color

    // MARK: - Convenience Methods

    func statusColor(for status: String) -> Color {
        switch status {
        case "active": statusActive
        case "waiting_for_input": statusWaitingInput
        case "waiting_for_permission": statusWaitingPermission
        case "idle": statusIdle
        case "ended": statusEnded
        default: .secondary
        }
    }

    func platformColor(for platform: String) -> Color {
        switch platform.lowercased() {
        case "mac", "macos", "darwin": platformMac
        case "linux": platformLinux
        case "windows": platformWindows
        default: platformDefault
        }
    }

    func eventColor(for hookEventName: String) -> Color {
        switch hookEventName {
        case "SessionStart": eventSessionStart
        case "SessionEnd": eventSessionEnd
        case "Stop": eventStop
        case "Notification": eventNotification
        case "UserPromptSubmit": eventUserPromptSubmit
        default: eventDefault
        }
    }

    // MARK: - Opacity Helpers

    func badgeBackground(_ color: Color) -> Color {
        color.opacity(0.1)
    }

    func iconBackground(_ color: Color) -> Color {
        color.opacity(0.12)
    }

    func statusHalo(_ color: Color) -> Color {
        color.opacity(0.3)
    }

    func notificationBadgeBackground() -> Color {
        eventNotification.opacity(0.15)
    }

    func groupContainerBackground() -> Color {
        cardBackground.opacity(0.3)
    }

    // MARK: - Adaptive Color Helper

    static func adaptive(light: UIColor, dark: UIColor) -> Color {
        Color(UIColor { $0.userInterfaceStyle == .dark ? dark : light })
    }

    // MARK: - Constants

    static let cardCornerRadius: CGFloat = 12
    static let cardBorderOpacity: Double = 0.3
    static let cardBorderWidth: CGFloat = 0.5
}
