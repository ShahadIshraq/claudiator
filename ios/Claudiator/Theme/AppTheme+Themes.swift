import SwiftUI

extension AppTheme {
    static let allThemes: [AppTheme] = [standard, neonOps, solarized, arctic]

    // MARK: - Standard
    static let standard = AppTheme(
        id: "standard",
        name: "Standard",
        preview: [.green, .blue, .orange, .red],
        statusActive: .green,
        statusWaitingInput: .orange,
        statusWaitingPermission: .red,
        statusIdle: .gray,
        statusEnded: .gray,
        platformMac: .blue,
        platformLinux: .orange,
        platformWindows: .cyan,
        platformDefault: .secondary,
        eventSessionStart: .green,
        eventSessionEnd: .gray,
        eventStop: .orange,
        eventNotification: adaptive(
            light: UIColor.systemRed,
            dark: UIColor(red: 1.0, green: 0.4, blue: 0.4, alpha: 1)
        ),
        eventUserPromptSubmit: .blue,
        eventDefault: .secondary,
        serverConnected: .green,
        serverDisconnected: .red,
        uiError: .red,
        uiAccent: .blue,
        uiTint: .blue,
        pageBackground: adaptive(
            light: UIColor.systemGroupedBackground,
            dark: UIColor.systemBackground
        ),
        cardBackground: adaptive(
            light: UIColor.secondarySystemGroupedBackground,
            dark: UIColor(white: 0.11, alpha: 1)
        ),
        cardBorder: adaptive(
            light: UIColor(white: 0.8, alpha: 1),
            dark: UIColor(white: 0.2, alpha: 1)
        )
    )

    // MARK: - Neon Ops
    static let neonOps = AppTheme(
        id: "neon_ops",
        name: "Neon Ops",
        preview: [Color(hex: 0x00FF99), Color(hex: 0xFF4080), Color(hex: 0xFFC200), Color(hex: 0x00BFFF)],
        statusActive: Color(hex: 0x00FF99),
        statusWaitingInput: Color(hex: 0xFFC200),
        statusWaitingPermission: Color(hex: 0xFF4080),
        statusIdle: Color(hex: 0x888888),
        statusEnded: Color(hex: 0x666666),
        platformMac: Color(hex: 0x00BFFF),
        platformLinux: Color(hex: 0xFFC200),
        platformWindows: Color(hex: 0x00FF99),
        platformDefault: Color(hex: 0x888888),
        eventSessionStart: Color(hex: 0x00FF99),
        eventSessionEnd: Color(hex: 0x666666),
        eventStop: Color(hex: 0xFFC200),
        eventNotification: adaptive(
            light: UIColor(red: 1.0, green: 0.25, blue: 0.5, alpha: 1),
            dark: UIColor(red: 1.0, green: 0.4, blue: 0.6, alpha: 1)
        ),
        eventUserPromptSubmit: Color(hex: 0x00BFFF),
        eventDefault: Color(hex: 0x888888),
        serverConnected: Color(hex: 0x00FF99),
        serverDisconnected: Color(hex: 0xFF4080),
        uiError: Color(hex: 0xFF4080),
        uiAccent: Color(hex: 0x00BFFF),
        uiTint: Color(hex: 0x00FF99),
        pageBackground: adaptive(
            light: UIColor(red: 0.88, green: 0.95, blue: 0.91, alpha: 1),
            dark: UIColor(red: 0.02, green: 0.05, blue: 0.03, alpha: 1)
        ),
        cardBackground: adaptive(
            light: UIColor(red: 0.97, green: 1.0, blue: 0.98, alpha: 1),
            dark: UIColor(red: 0.06, green: 0.12, blue: 0.08, alpha: 1)
        ),
        cardBorder: adaptive(
            light: UIColor(red: 0.0, green: 0.8, blue: 0.5, alpha: 1),
            dark: UIColor(red: 0.0, green: 0.6, blue: 0.35, alpha: 1)
        )
    )

    // MARK: - Solarized
    static let solarized = AppTheme(
        id: "solarized",
        name: "Solarized",
        preview: [Color(hex: 0x859900), Color(hex: 0x268BD2), Color(hex: 0xB58900), Color(hex: 0xDC322F)],
        statusActive: Color(hex: 0x859900),
        statusWaitingInput: Color(hex: 0xB58900),
        statusWaitingPermission: Color(hex: 0xDC322F),
        statusIdle: Color(hex: 0x93A1A1),
        statusEnded: Color(hex: 0x839496),
        platformMac: Color(hex: 0x268BD2),
        platformLinux: Color(hex: 0xCB4B16),
        platformWindows: Color(hex: 0x2AA198),
        platformDefault: Color(hex: 0x93A1A1),
        eventSessionStart: Color(hex: 0x859900),
        eventSessionEnd: Color(hex: 0x839496),
        eventStop: Color(hex: 0xCB4B16),
        eventNotification: adaptive(
            light: UIColor(red: 0.86, green: 0.20, blue: 0.18, alpha: 1),
            dark: UIColor(red: 0.95, green: 0.35, blue: 0.33, alpha: 1)
        ),
        eventUserPromptSubmit: Color(hex: 0x268BD2),
        eventDefault: Color(hex: 0x93A1A1),
        serverConnected: Color(hex: 0x859900),
        serverDisconnected: Color(hex: 0xDC322F),
        uiError: Color(hex: 0xDC322F),
        uiAccent: Color(hex: 0x268BD2),
        uiTint: Color(hex: 0x268BD2),
        pageBackground: adaptive(
            light: UIColor(red: 0.89, green: 0.87, blue: 0.79, alpha: 1),
            dark: UIColor(red: 0.0, green: 0.13, blue: 0.16, alpha: 1)
        ),
        cardBackground: adaptive(
            light: UIColor(red: 0.98, green: 0.96, blue: 0.89, alpha: 1),
            dark: UIColor(red: 0.03, green: 0.21, blue: 0.26, alpha: 1)
        ),
        cardBorder: adaptive(
            light: UIColor(red: 0.82, green: 0.78, blue: 0.69, alpha: 1),
            dark: UIColor(red: 0.07, green: 0.29, blue: 0.34, alpha: 1)
        )
    )

    // MARK: - Arctic
    static let arctic = AppTheme(
        id: "arctic",
        name: "Arctic",
        preview: [Color(hex: 0x2EB887), Color(hex: 0x3B82F6), Color(hex: 0x8B95A5), Color(hex: 0xEF4444)],
        statusActive: Color(hex: 0x2EB887),
        statusWaitingInput: Color(hex: 0xF59E0B),
        statusWaitingPermission: Color(hex: 0xEF4444),
        statusIdle: Color(hex: 0x8B95A5),
        statusEnded: Color(hex: 0x6B7280),
        platformMac: Color(hex: 0x3B82F6),
        platformLinux: Color(hex: 0xF59E0B),
        platformWindows: Color(hex: 0x06B6D4),
        platformDefault: Color(hex: 0x8B95A5),
        eventSessionStart: Color(hex: 0x2EB887),
        eventSessionEnd: Color(hex: 0x6B7280),
        eventStop: Color(hex: 0xF59E0B),
        eventNotification: adaptive(
            light: UIColor(red: 0.94, green: 0.27, blue: 0.27, alpha: 1),
            dark: UIColor(red: 1.0, green: 0.45, blue: 0.45, alpha: 1)
        ),
        eventUserPromptSubmit: Color(hex: 0x3B82F6),
        eventDefault: Color(hex: 0x8B95A5),
        serverConnected: Color(hex: 0x2EB887),
        serverDisconnected: Color(hex: 0xEF4444),
        uiError: Color(hex: 0xEF4444),
        uiAccent: Color(hex: 0x3B82F6),
        uiTint: Color(hex: 0x2EB887),
        pageBackground: adaptive(
            light: UIColor(red: 0.87, green: 0.92, blue: 0.98, alpha: 1),
            dark: UIColor(red: 0.03, green: 0.06, blue: 0.10, alpha: 1)
        ),
        cardBackground: adaptive(
            light: UIColor(red: 0.96, green: 0.98, blue: 1.0, alpha: 1),
            dark: UIColor(red: 0.08, green: 0.13, blue: 0.20, alpha: 1)
        ),
        cardBorder: adaptive(
            light: UIColor(red: 0.78, green: 0.86, blue: 0.93, alpha: 1),
            dark: UIColor(red: 0.13, green: 0.19, blue: 0.27, alpha: 1)
        )
    )
}

// MARK: - Color Hex Init

extension Color {
    init(hex: UInt, opacity: Double = 1.0) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255,
            opacity: opacity
        )
    }
}
