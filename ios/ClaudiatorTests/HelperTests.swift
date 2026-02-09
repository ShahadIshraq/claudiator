import Testing
import Foundation
@testable import Claudiator

@Suite("Helper Function Tests")
struct HelperTests {
    // MARK: - relativeTime

    @Test("relativeTime with recent timestamp")
    func testRelativeTimeRecent() async {
        let now = Date()
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        let isoString = formatter.string(from: now.addingTimeInterval(-30))

        let result = await MainActor.run {
            relativeTime(isoString)
        }
        // Result should not be the original string (should be formatted)
        #expect(result != isoString)
        #expect(!result.isEmpty)
    }

    @Test("relativeTime with invalid timestamp returns original")
    func testRelativeTimeInvalid() async {
        let invalid = "not-a-date"
        let result = await MainActor.run {
            relativeTime(invalid)
        }
        #expect(result == invalid)
    }

    @Test("relativeTime without fractional seconds")
    func testRelativeTimeNoFractional() async {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime]
        let isoString = formatter.string(from: Date().addingTimeInterval(-60))

        let result = await MainActor.run {
            relativeTime(isoString)
        }
        // Result should not be the original string (should be formatted)
        #expect(result != isoString)
        #expect(!result.isEmpty)
    }

    // MARK: - cwdShortDisplay

    @Test("cwdShortDisplay with Unix path")
    func testCwdShortDisplayUnix() {
        let cwd = "/Users/test/workspace/project"
        let result = cwdShortDisplay(cwd)
        #expect(result == "workspace/project")
    }

    @Test("cwdShortDisplay with single component")
    func testCwdShortDisplaySingle() {
        let cwd = "/project"
        let result = cwdShortDisplay(cwd)
        #expect(result == cwd)
    }

    @Test("cwdShortDisplay with Windows path")
    func testCwdShortDisplayWindows() {
        let cwd = "C:\\Users\\test\\Documents\\project"
        let result = cwdShortDisplay(cwd)
        #expect(result == "Documents/project")
    }

    @Test("cwdShortDisplay with exactly two components")
    func testCwdShortDisplayTwoComponents() {
        let cwd = "/workspace/project"
        let result = cwdShortDisplay(cwd)
        #expect(result == "workspace/project")
    }

    // MARK: - statusDisplayLabel

    @Test("statusDisplayLabel for known statuses")
    func testStatusDisplayLabelKnown() {
        #expect(statusDisplayLabel("active") == "Active")
        #expect(statusDisplayLabel("waiting_for_input") == "Waiting for Input")
        #expect(statusDisplayLabel("waiting_for_permission") == "Waiting for Permission")
        #expect(statusDisplayLabel("idle") == "Idle")
        #expect(statusDisplayLabel("ended") == "Ended")
    }

    @Test("statusDisplayLabel for unknown status")
    func testStatusDisplayLabelUnknown() {
        let result = statusDisplayLabel("custom_status")
        #expect(result == "Custom Status")
    }

    @Test("statusDisplayLabel is case sensitive for known statuses")
    func testStatusDisplayLabelCaseSensitive() {
        // statusDisplayLabel is case sensitive - uppercase "ACTIVE" won't match lowercase "active"
        let result = statusDisplayLabel("ACTIVE")
        // Should be treated as unknown and capitalized
        #expect(result == "Active")
    }

    // MARK: - priorityStatus

    @Test("priorityStatus returns waiting_for_permission when present")
    func testPriorityStatusWaitingPermission() {
        let sessions = [
            Session(
                sessionId: "1",
                deviceId: "dev1",
                startedAt: "2024-01-15T10:00:00Z",
                lastEvent: "2024-01-15T11:00:00Z",
                status: "idle",
                cwd: nil,
                title: nil,
                deviceName: nil,
                platform: nil
            ),
            Session(
                sessionId: "2",
                deviceId: "dev1",
                startedAt: "2024-01-15T10:00:00Z",
                lastEvent: "2024-01-15T11:00:00Z",
                status: "waiting_for_permission",
                cwd: nil,
                title: nil,
                deviceName: nil,
                platform: nil
            ),
            Session(
                sessionId: "3",
                deviceId: "dev1",
                startedAt: "2024-01-15T10:00:00Z",
                lastEvent: "2024-01-15T11:00:00Z",
                status: "active",
                cwd: nil,
                title: nil,
                deviceName: nil,
                platform: nil
            ),
        ]

        let result = priorityStatus(for: sessions)
        #expect(result == "waiting_for_permission")
    }

    @Test("priorityStatus returns waiting_for_input when no permission wait")
    func testPriorityStatusWaitingInput() {
        let sessions = [
            Session(
                sessionId: "1",
                deviceId: "dev1",
                startedAt: "2024-01-15T10:00:00Z",
                lastEvent: "2024-01-15T11:00:00Z",
                status: "idle",
                cwd: nil,
                title: nil,
                deviceName: nil,
                platform: nil
            ),
            Session(
                sessionId: "2",
                deviceId: "dev1",
                startedAt: "2024-01-15T10:00:00Z",
                lastEvent: "2024-01-15T11:00:00Z",
                status: "waiting_for_input",
                cwd: nil,
                title: nil,
                deviceName: nil,
                platform: nil
            ),
        ]

        let result = priorityStatus(for: sessions)
        #expect(result == "waiting_for_input")
    }

    @Test("priorityStatus returns active when no waiting states")
    func testPriorityStatusActive() {
        let sessions = [
            Session(
                sessionId: "1",
                deviceId: "dev1",
                startedAt: "2024-01-15T10:00:00Z",
                lastEvent: "2024-01-15T11:00:00Z",
                status: "active",
                cwd: nil,
                title: nil,
                deviceName: nil,
                platform: nil
            ),
            Session(
                sessionId: "2",
                deviceId: "dev1",
                startedAt: "2024-01-15T10:00:00Z",
                lastEvent: "2024-01-15T11:00:00Z",
                status: "idle",
                cwd: nil,
                title: nil,
                deviceName: nil,
                platform: nil
            ),
        ]

        let result = priorityStatus(for: sessions)
        #expect(result == "active")
    }

    @Test("priorityStatus returns idle when only idle and ended")
    func testPriorityStatusIdle() {
        let sessions = [
            Session(
                sessionId: "1",
                deviceId: "dev1",
                startedAt: "2024-01-15T10:00:00Z",
                lastEvent: "2024-01-15T11:00:00Z",
                status: "idle",
                cwd: nil,
                title: nil,
                deviceName: nil,
                platform: nil
            ),
            Session(
                sessionId: "2",
                deviceId: "dev1",
                startedAt: "2024-01-15T10:00:00Z",
                lastEvent: "2024-01-15T11:00:00Z",
                status: "ended",
                cwd: nil,
                title: nil,
                deviceName: nil,
                platform: nil
            ),
        ]

        let result = priorityStatus(for: sessions)
        #expect(result == "idle")
    }

    @Test("priorityStatus returns ended for all ended sessions")
    func testPriorityStatusEnded() {
        let sessions = [
            Session(
                sessionId: "1",
                deviceId: "dev1",
                startedAt: "2024-01-15T10:00:00Z",
                lastEvent: "2024-01-15T11:00:00Z",
                status: "ended",
                cwd: nil,
                title: nil,
                deviceName: nil,
                platform: nil
            ),
        ]

        let result = priorityStatus(for: sessions)
        #expect(result == "ended")
    }

    @Test("priorityStatus returns ended for empty array")
    func testPriorityStatusEmpty() {
        let sessions: [Session] = []
        let result = priorityStatus(for: sessions)
        #expect(result == "ended")
    }

    // MARK: - platformImageName

    @Test("platformImageName returns correct image for platforms")
    func testPlatformImageName() {
        #expect(platformImageName("mac") == "AppleLogo")
        #expect(platformImageName("macos") == "AppleLogo")
        #expect(platformImageName("darwin") == "AppleLogo")
        #expect(platformImageName("linux") == "LinuxLogo")
        #expect(platformImageName("windows") == "WindowsLogo")
        #expect(platformImageName("unknown") == "WindowsLogo")
    }

    @Test("platformImageName is case insensitive")
    func testPlatformImageNameCaseInsensitive() {
        #expect(platformImageName("MAC") == "AppleLogo")
        #expect(platformImageName("Linux") == "LinuxLogo")
        #expect(platformImageName("WINDOWS") == "WindowsLogo")
    }
}
