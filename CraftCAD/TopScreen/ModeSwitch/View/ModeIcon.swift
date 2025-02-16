import SwiftUI

struct ModeIcon: View {
    let mode: AppMode  // ← 修正
    let isSelected: Bool
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            VStack {
                Image(systemName: iconName)
                    .resizable()
                    .scaledToFit()
                    .frame(width: 50, height: 50)
                    .modifier(ModeHighlight(isSelected: isSelected))
                Text(mode.rawValue)
                    .font(.caption)
            }
        }
        .buttonStyle(PlainButtonStyle())
    }
    
    private var iconName: String {
        switch mode {
        case .diy: return "hammer"
        case .leatherCraft: return "scissors"
        case .pro: return "wrench"
        }
    }
}
