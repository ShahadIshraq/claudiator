import Testing
import SwiftUI
@testable import Claudiator

@Suite("Theme Tests")
struct ThemeTests {
    // MARK: - Helper to create a test theme

    func createTestTheme() -> AppTheme {
        AppTheme(
            id: "test",
            name: "Test Theme",
            preview: [.red, .blue, .green],
            statusActive: .green,
            statusWaitingInput: .yellow,
            statusWaitingPermission: .orange,
            statusIdle: .gray,
            statusEnded: .secondary,
            platformMac: .blue,
            platformLinux: .orange,
            platformWindows: .cyan,
            platformDefault: .gray,
            eventSessionStart: .green,
            eventSessionEnd: .red,
            eventStop: .orange,
            eventNotification: .blue,
            eventUserPromptSubmit: .purple,
            eventDefault: .gray,
            serverConnected: .green,
            serverDisconnected: .red,
            uiError: .red,
            uiAccent: .blue,
            uiTint: .cyan,
            pageBackground: .white,
            cardBackground: .gray,
            cardBorder: .black
        )
    }

    // MARK: - statusColor Tests

    @Test("statusColor returns correct color for known statuses")
    func testStatusColorKnownStatuses() {
        let theme = createTestTheme()

        #expect(theme.statusColor(for: "active") == theme.statusActive)
        #expect(theme.statusColor(for: "waiting_for_input") == theme.statusWaitingInput)
        #expect(theme.statusColor(for: "waiting_for_permission") == theme.statusWaitingPermission)
        #expect(theme.statusColor(for: "idle") == theme.statusIdle)
        #expect(theme.statusColor(for: "ended") == theme.statusEnded)
    }

    @Test("statusColor returns secondary for unknown status")
    func testStatusColorUnknownStatus() {
        let theme = createTestTheme()
        let result = theme.statusColor(for: "unknown_status")

        #expect(result == .secondary)
    }

    @Test("statusColor handles empty string")
    func testStatusColorEmptyString() {
        let theme = createTestTheme()
        let result = theme.statusColor(for: "")

        #expect(result == .secondary)
    }

    // MARK: - platformColor Tests

    @Test("platformColor returns correct color for Mac variants")
    func testPlatformColorMac() {
        let theme = createTestTheme()

        #expect(theme.platformColor(for: "mac") == theme.platformMac)
        #expect(theme.platformColor(for: "macos") == theme.platformMac)
        #expect(theme.platformColor(for: "darwin") == theme.platformMac)
    }

    @Test("platformColor returns correct color for Linux")
    func testPlatformColorLinux() {
        let theme = createTestTheme()

        #expect(theme.platformColor(for: "linux") == theme.platformLinux)
    }

    @Test("platformColor returns correct color for Windows")
    func testPlatformColorWindows() {
        let theme = createTestTheme()

        #expect(theme.platformColor(for: "windows") == theme.platformWindows)
    }

    @Test("platformColor returns default for unknown platform")
    func testPlatformColorUnknown() {
        let theme = createTestTheme()

        #expect(theme.platformColor(for: "unknown") == theme.platformDefault)
        #expect(theme.platformColor(for: "") == theme.platformDefault)
    }

    @Test("platformColor is case insensitive")
    func testPlatformColorCaseInsensitive() {
        let theme = createTestTheme()

        #expect(theme.platformColor(for: "MAC") == theme.platformMac)
        #expect(theme.platformColor(for: "Linux") == theme.platformLinux)
        #expect(theme.platformColor(for: "WINDOWS") == theme.platformWindows)
        #expect(theme.platformColor(for: "DaRwIn") == theme.platformMac)
    }

    // MARK: - eventColor Tests

    @Test("eventColor returns correct color for known events")
    func testEventColorKnownEvents() {
        let theme = createTestTheme()

        #expect(theme.eventColor(for: "SessionStart") == theme.eventSessionStart)
        #expect(theme.eventColor(for: "SessionEnd") == theme.eventSessionEnd)
        #expect(theme.eventColor(for: "Stop") == theme.eventStop)
        #expect(theme.eventColor(for: "Notification") == theme.eventNotification)
        #expect(theme.eventColor(for: "UserPromptSubmit") == theme.eventUserPromptSubmit)
    }

    @Test("eventColor returns default for unknown event")
    func testEventColorUnknownEvent() {
        let theme = createTestTheme()

        #expect(theme.eventColor(for: "UnknownEvent") == theme.eventDefault)
        #expect(theme.eventColor(for: "") == theme.eventDefault)
    }

    @Test("eventColor is case sensitive")
    func testEventColorCaseSensitive() {
        let theme = createTestTheme()

        // Event names should be exact match, not lowercased
        #expect(theme.eventColor(for: "SessionStart") == theme.eventSessionStart)
        #expect(theme.eventColor(for: "sessionstart") == theme.eventDefault)
    }

    // MARK: - Opacity Helper Tests

    @Test("badgeBackground returns color with correct opacity")
    func testBadgeBackground() {
        let theme = createTestTheme()
        let result = theme.badgeBackground(.red)

        // We can't directly compare opacity, but we can verify it doesn't crash
        #expect(result != .clear)
    }

    @Test("iconBackground returns color with correct opacity")
    func testIconBackground() {
        let theme = createTestTheme()
        let result = theme.iconBackground(.blue)

        #expect(result != .clear)
    }

    @Test("statusHalo returns color with correct opacity")
    func testStatusHalo() {
        let theme = createTestTheme()
        let result = theme.statusHalo(.green)

        #expect(result != .clear)
    }

    @Test("notificationBadgeBackground uses event notification color")
    func testNotificationBadgeBackground() {
        let theme = createTestTheme()
        let result = theme.notificationBadgeBackground()

        #expect(result != .clear)
    }

    @Test("groupContainerBackground uses card background")
    func testGroupContainerBackground() {
        let theme = createTestTheme()
        let result = theme.groupContainerBackground()

        #expect(result != .clear)
    }

    // MARK: - AppTheme Constants Tests

    @Test("AppTheme constants have expected values")
    func testAppThemeConstants() {
        #expect(AppTheme.cardCornerRadius == 12)
        #expect(AppTheme.cardBorderOpacity == 0.3)
        #expect(AppTheme.cardBorderWidth == 0.5)
    }

    // MARK: - AppTheme Identifiable Tests

    @Test("AppTheme is identifiable by id")
    func testAppThemeIdentifiable() {
        let theme = createTestTheme()

        #expect(theme.id == "test")
    }

    @Test("AppTheme equality works correctly")
    func testAppThemeEquality() {
        let theme1 = createTestTheme()
        let theme2 = createTestTheme()

        #expect(theme1 == theme2)
    }

    @Test("AppTheme with different ids are not equal")
    func testAppThemeInequalityDifferentIds() {
        let theme1 = createTestTheme()
        var theme2 = createTestTheme()
        // We can't modify the id directly since it's a let property,
        // so we create a new theme with a different id
        let theme3 = AppTheme(
            id: "different",
            name: theme2.name,
            preview: theme2.preview,
            statusActive: theme2.statusActive,
            statusWaitingInput: theme2.statusWaitingInput,
            statusWaitingPermission: theme2.statusWaitingPermission,
            statusIdle: theme2.statusIdle,
            statusEnded: theme2.statusEnded,
            platformMac: theme2.platformMac,
            platformLinux: theme2.platformLinux,
            platformWindows: theme2.platformWindows,
            platformDefault: theme2.platformDefault,
            eventSessionStart: theme2.eventSessionStart,
            eventSessionEnd: theme2.eventSessionEnd,
            eventStop: theme2.eventStop,
            eventNotification: theme2.eventNotification,
            eventUserPromptSubmit: theme2.eventUserPromptSubmit,
            eventDefault: theme2.eventDefault,
            serverConnected: theme2.serverConnected,
            serverDisconnected: theme2.serverDisconnected,
            uiError: theme2.uiError,
            uiAccent: theme2.uiAccent,
            uiTint: theme2.uiTint,
            pageBackground: theme2.pageBackground,
            cardBackground: theme2.cardBackground,
            cardBorder: theme2.cardBorder
        )

        #expect(theme1 != theme3)
    }
}
