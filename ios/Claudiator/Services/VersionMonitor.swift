import Foundation
import Observation

@Observable
class VersionMonitor {
    private(set) var dataVersion: UInt64 = 0
    private var task: Task<Void, Never>?

    func start(apiClient: APIClient) {
        guard task == nil else { return }
        task = Task {
            while !Task.isCancelled {
                if let version = try? await apiClient.ping() {
                    await MainActor.run { self.dataVersion = version }
                }
                try? await Task.sleep(for: .seconds(10))
            }
        }
    }

    func stop() {
        task?.cancel()
        task = nil
    }
}
