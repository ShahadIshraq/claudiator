import SwiftUI

enum AppearanceMode: String, CaseIterable {
    case system = "System"
    case light = "Light"
    case dark = "Dark"

    var colorScheme: ColorScheme? {
        switch self {
        case .system: nil
        case .light: .light
        case .dark: .dark
        }
    }
}

@MainActor
@Observable
class ThemeManager {
    private static let themeKey = "selectedThemeId"
    private static let appearanceKey = "appearanceMode"

    var current: AppTheme {
        didSet {
            UserDefaults.standard.set(current.id, forKey: Self.themeKey)
        }
    }

    var appearance: AppearanceMode {
        didSet {
            UserDefaults.standard.set(appearance.rawValue, forKey: Self.appearanceKey)
        }
    }

    init() {
        let savedId = UserDefaults.standard.string(forKey: Self.themeKey) ?? "standard"
        self.current = AppTheme.allThemes.first { $0.id == savedId } ?? .standard

        let savedAppearance = UserDefaults.standard.string(forKey: Self.appearanceKey) ?? "System"
        self.appearance = AppearanceMode(rawValue: savedAppearance) ?? .system
    }

    func select(_ theme: AppTheme) {
        current = theme
    }

    func setAppearance(_ mode: AppearanceMode) {
        appearance = mode
    }
}
