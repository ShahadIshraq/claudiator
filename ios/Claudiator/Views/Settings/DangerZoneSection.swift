import SwiftUI

struct DangerZoneSection: View {
    @Environment(APIClient.self) private var apiClient
    @State private var showDisconnectConfirm = false

    var body: some View {
        Section {
            Button(role: .destructive) {
                showDisconnectConfirm = true
            } label: {
                HStack {
                    Spacer()
                    Text("Disconnect")
                    Spacer()
                }
            }
            .themedCard()
            .confirmationDialog("Disconnect from server?", isPresented: $showDisconnectConfirm, titleVisibility: .visible) {
                Button("Disconnect", role: .destructive) {
                    apiClient.disconnect()
                }
                Button("Cancel", role: .cancel) {}
            } message: {
                Text("This will remove your server URL and API key. You'll need to re-enter them to reconnect.")
            }
        }
    }
}
