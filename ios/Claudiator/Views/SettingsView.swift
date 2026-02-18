import SwiftUI

struct SettingsView: View {
    var body: some View {
        List {
            Section {
                HStack(spacing: 14) {
                    Image("ClaudiatorLogo")
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(width: 52, height: 52)
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Claudiator")
                            .font(.title3)
                            .fontWeight(.bold)
                        Text("v1.0.0")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                .padding(.vertical, 4)
                .listRowBackground(Color.clear)
            }

            AppearanceSection()

            ServerConfigSection()

            DangerZoneSection()
        }
        .themedPage()
        .scrollDismissesKeyboard(.interactively)
        .toolbar {
            ToolbarItemGroup(placement: .keyboard) {
                Spacer()
                Button("Done") {
                    UIApplication.shared.sendAction(
                        #selector(UIResponder.resignFirstResponder),
                        to: nil,
                        from: nil,
                        for: nil
                    )
                }
            }
        }
        .navigationTitle("Settings")
    }
}
