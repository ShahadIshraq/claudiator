import Testing
import Foundation
@testable import Claudiator

@Suite("ViewModel Tests")
@MainActor
struct ViewModelTests {
    // MARK: - Helper Methods

    func createTestSession(
        id: String,
        deviceId: String,
        status: String,
        cwd: String? = nil
    ) -> Session {
        Session(
            sessionId: id,
            deviceId: deviceId,
            startedAt: "2024-01-15T10:00:00Z",
            lastEvent: "2024-01-15T11:00:00Z",
            status: status,
            cwd: cwd,
            title: nil,
            deviceName: nil,
            platform: nil
        )
    }

    // MARK: - AllSessionsViewModel Tests

    @Test("AllSessionsViewModel initializes with correct defaults")
    func testAllSessionsViewModelInit() {
        let viewModel = AllSessionsViewModel()

        #expect(viewModel.sessions.isEmpty)
        #expect(viewModel.isLoading == false)
        #expect(viewModel.errorMessage == nil)
        #expect(viewModel.filter == .active)
        #expect(viewModel.groupedSessions.isEmpty)
        #expect(viewModel.expandedDevices.isEmpty)
    }

    @Test("AllSessionsViewModel toggleGrouping changes state")
    func testToggleGrouping() {
        let viewModel = AllSessionsViewModel()
        let initialState = viewModel.isGroupedByDevice

        viewModel.toggleGrouping()
        #expect(viewModel.isGroupedByDevice != initialState)

        viewModel.toggleGrouping()
        #expect(viewModel.isGroupedByDevice == initialState)
    }

    @Test("AllSessionsViewModel toggleDevice expands and collapses")
    func testToggleDevice() {
        let viewModel = AllSessionsViewModel()
        let deviceId = "dev1"

        #expect(!viewModel.expandedDevices.contains(deviceId))

        viewModel.toggleDevice(deviceId)
        #expect(viewModel.expandedDevices.contains(deviceId))

        viewModel.toggleDevice(deviceId)
        #expect(!viewModel.expandedDevices.contains(deviceId))
    }

    @Test("AllSessionsViewModel filter enum has correct cases")
    func testSessionFilterCases() {
        let allCases = AllSessionsViewModel.SessionFilter.allCases

        #expect(allCases.count == 2)
        #expect(allCases.contains(.active))
        #expect(allCases.contains(.all))

        #expect(AllSessionsViewModel.SessionFilter.active.rawValue == "Active")
        #expect(AllSessionsViewModel.SessionFilter.all.rawValue == "All")
    }

    // MARK: - DeviceListViewModel Tests

    @Test("DeviceListViewModel initializes with correct defaults")
    func testDeviceListViewModelInit() {
        let viewModel = DeviceListViewModel()

        #expect(viewModel.devices.isEmpty)
        #expect(viewModel.statusCounts.isEmpty)
        #expect(viewModel.isLoading == false)
        #expect(viewModel.errorMessage == nil)
    }

    // MARK: - SessionStatusCounts Tests

    @Test("SessionStatusCounts initializes with zeros")
    func testSessionStatusCountsInit() {
        let counts = SessionStatusCounts()

        #expect(counts.active == 0)
        #expect(counts.waitingInput == 0)
        #expect(counts.waitingPermission == 0)
        #expect(counts.idle == 0)
        #expect(counts.ended == 0)
        #expect(counts.totalActive == 0)
    }

    @Test("SessionStatusCounts totalActive excludes ended")
    func testSessionStatusCountsTotalActive() {
        var counts = SessionStatusCounts()
        counts.active = 2
        counts.waitingInput = 1
        counts.waitingPermission = 1
        counts.idle = 3
        counts.ended = 5

        #expect(counts.totalActive == 7)
    }

    @Test("SessionStatusCounts totalActive with only ended")
    func testSessionStatusCountsTotalActiveOnlyEnded() {
        var counts = SessionStatusCounts()
        counts.ended = 10

        #expect(counts.totalActive == 0)
    }

    @Test("SessionStatusCounts totalActive with all types")
    func testSessionStatusCountsTotalActiveAllTypes() {
        var counts = SessionStatusCounts()
        counts.active = 1
        counts.waitingInput = 1
        counts.waitingPermission = 1
        counts.idle = 1
        counts.ended = 1

        #expect(counts.totalActive == 4)
    }

    @Test("SessionStatusCounts can be mutated")
    func testSessionStatusCountsMutation() {
        var counts = SessionStatusCounts()
        #expect(counts.active == 0)

        counts.active = 5
        #expect(counts.active == 5)

        counts.active += 1
        #expect(counts.active == 6)
    }

    // MARK: - Session Grouping Logic Tests

    @Test("Sessions can be grouped by device ID")
    func testSessionGroupingLogic() {
        let sessions = [
            createTestSession(id: "sess1", deviceId: "dev1", status: "active"),
            createTestSession(id: "sess2", deviceId: "dev1", status: "idle"),
            createTestSession(id: "sess3", deviceId: "dev2", status: "active"),
        ]

        let grouped = Dictionary(grouping: sessions) { $0.deviceId }

        #expect(grouped.count == 2)
        #expect(grouped["dev1"]?.count == 2)
        #expect(grouped["dev2"]?.count == 1)
    }

    @Test("Active session detection logic")
    func testActiveSessionDetection() {
        let sessions = [
            createTestSession(id: "sess1", deviceId: "dev1", status: "active"),
            createTestSession(id: "sess2", deviceId: "dev1", status: "ended"),
        ]

        let hasActive = sessions.contains { $0.status != "ended" }
        #expect(hasActive == true)

        let endedSessions = [
            createTestSession(id: "sess1", deviceId: "dev1", status: "ended"),
            createTestSession(id: "sess2", deviceId: "dev1", status: "ended"),
        ]

        let hasActiveEnded = endedSessions.contains { $0.status != "ended" }
        #expect(hasActiveEnded == false)
    }

    // MARK: - Status Count Aggregation Tests

    @Test("Status count aggregation logic")
    func testStatusCountAggregation() {
        let sessions = [
            createTestSession(id: "sess1", deviceId: "dev1", status: "active"),
            createTestSession(id: "sess2", deviceId: "dev1", status: "active"),
            createTestSession(id: "sess3", deviceId: "dev1", status: "waiting_for_input"),
            createTestSession(id: "sess4", deviceId: "dev1", status: "waiting_for_permission"),
            createTestSession(id: "sess5", deviceId: "dev1", status: "idle"),
            createTestSession(id: "sess6", deviceId: "dev1", status: "ended"),
        ]

        var counts = SessionStatusCounts()
        for session in sessions where session.deviceId == "dev1" {
            switch session.status {
            case "active": counts.active += 1
            case "waiting_for_input": counts.waitingInput += 1
            case "waiting_for_permission": counts.waitingPermission += 1
            case "idle": counts.idle += 1
            case "ended": counts.ended += 1
            default: break
            }
        }

        #expect(counts.active == 2)
        #expect(counts.waitingInput == 1)
        #expect(counts.waitingPermission == 1)
        #expect(counts.idle == 1)
        #expect(counts.ended == 1)
        #expect(counts.totalActive == 5)
    }

    @Test("Multiple devices status aggregation")
    func testMultipleDevicesAggregation() {
        let sessions = [
            createTestSession(id: "sess1", deviceId: "dev1", status: "active"),
            createTestSession(id: "sess2", deviceId: "dev1", status: "idle"),
            createTestSession(id: "sess3", deviceId: "dev2", status: "active"),
            createTestSession(id: "sess4", deviceId: "dev2", status: "waiting_for_input"),
        ]

        var counts: [String: SessionStatusCounts] = [:]
        for session in sessions {
            var deviceCounts = counts[session.deviceId, default: SessionStatusCounts()]
            switch session.status {
            case "active": deviceCounts.active += 1
            case "waiting_for_input": deviceCounts.waitingInput += 1
            case "idle": deviceCounts.idle += 1
            default: break
            }
            counts[session.deviceId] = deviceCounts
        }

        #expect(counts.count == 2)
        #expect(counts["dev1"]?.active == 1)
        #expect(counts["dev1"]?.idle == 1)
        #expect(counts["dev2"]?.active == 1)
        #expect(counts["dev2"]?.waitingInput == 1)
    }
}
