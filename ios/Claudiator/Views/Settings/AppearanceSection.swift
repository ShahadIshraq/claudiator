import SwiftUI

struct AppearanceSection: View {
    @Environment(ThemeManager.self) private var themeManager

    var body: some View {
        Section("Appearance") {
            Picker("Mode", selection: Bindable(themeManager).appearance) {
                ForEach(AppearanceMode.allCases, id: \.self) { mode in
                    Text(mode.rawValue).tag(mode)
                }
            }
            .pickerStyle(.segmented)
            .listRowBackground(Color.clear)
        }

        Section {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 12) {
                    ForEach(AppTheme.allThemes) { theme in
                        ThemePreviewCard(
                            theme: theme,
                            isSelected: themeManager.current.id == theme.id
                        ) {
                            withAnimation(.easeInOut(duration: 0.2)) {
                                themeManager.select(theme)
                            }
                        }
                    }
                }
                .padding(.horizontal, 4)
                .padding(.vertical, 8)
            }
            .listRowInsets(EdgeInsets(top: 0, leading: 12, bottom: 0, trailing: 12))
            .themedCard()
        } header: {
            Text("Theme")
        }
    }
}

// MARK: - Theme Preview Card

private struct ThemePreviewCard: View {
    let theme: AppTheme
    let isSelected: Bool
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            VStack(spacing: 8) {
                // Mini mockup
                VStack(spacing: 0) {
                    // Mini nav bar
                    HStack {
                        RoundedRectangle(cornerRadius: 2)
                            .fill(Color.primary.opacity(0.6))
                            .frame(width: 40, height: 6)
                        Spacer()
                    }
                    .padding(.horizontal, 8)
                    .padding(.top, 8)
                    .padding(.bottom, 6)

                    // Mini device rows
                    VStack(spacing: 4) {
                        MiniDeviceRow(
                            platformColor: theme.platformMac,
                            statusColor: theme.statusActive,
                            statusWidth: 22,
                            cardColor: theme.cardBackground
                        )
                        MiniDeviceRow(
                            platformColor: theme.platformLinux,
                            statusColor: theme.statusWaitingInput,
                            statusWidth: 18,
                            cardColor: theme.cardBackground
                        )
                        MiniDeviceRow(
                            platformColor: theme.platformWindows,
                            statusColor: theme.statusIdle,
                            statusWidth: 14,
                            cardColor: theme.cardBackground
                        )
                    }
                    .padding(.horizontal, 6)
                    .padding(.bottom, 8)
                }
                .frame(width: 80, height: 82)
                .background(theme.pageBackground)
                .clipShape(RoundedRectangle(cornerRadius: 10))
                .overlay(
                    RoundedRectangle(cornerRadius: 10)
                        .strokeBorder(
                            isSelected ? theme.statusActive : Color.clear,
                            lineWidth: 2
                        )
                )

                // Label + radio
                Text(theme.name)
                    .font(.caption2)
                    .foregroundStyle(.primary)
                    .lineLimit(1)

                Circle()
                    .strokeBorder(isSelected ? theme.statusActive : Color.gray.opacity(0.4), lineWidth: isSelected ? 0 : 1.5)
                    .background(
                        Circle()
                            .fill(isSelected ? theme.statusActive : Color.clear)
                    )
                    .overlay(
                        isSelected ?
                            Image(systemName: "checkmark")
                            .font(.system(size: 8, weight: .bold))
                            .foregroundStyle(.white)
                            : nil
                    )
                    .frame(width: 18, height: 18)
            }
        }
        .buttonStyle(.plain)
    }
}

private struct MiniDeviceRow: View {
    let platformColor: Color
    let statusColor: Color
    let statusWidth: CGFloat
    let cardColor: Color

    var body: some View {
        HStack(spacing: 4) {
            RoundedRectangle(cornerRadius: 3)
                .fill(platformColor.opacity(0.15))
                .overlay(
                    RoundedRectangle(cornerRadius: 3)
                        .fill(platformColor)
                        .frame(width: 6, height: 6)
                )
                .frame(width: 16, height: 16)

            VStack(alignment: .leading, spacing: 2) {
                RoundedRectangle(cornerRadius: 1)
                    .fill(Color.primary.opacity(0.4))
                    .frame(width: 30, height: 4)
                HStack(spacing: 3) {
                    Circle()
                        .fill(statusColor)
                        .frame(width: 4, height: 4)
                    RoundedRectangle(cornerRadius: 1)
                        .fill(statusColor.opacity(0.4))
                        .frame(width: statusWidth, height: 3)
                }
            }
            Spacer()
        }
        .padding(4)
        .background(cardColor)
        .clipShape(RoundedRectangle(cornerRadius: 5))
    }
}
