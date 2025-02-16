import SwiftUI

struct ModeSelector: View {
    @EnvironmentObject var modeManager: ModeManager
    
    var body: some View {
        HStack(spacing: 20) {
            ForEach(AppMode.allCases, id: \.self) { mode in  // ← 修正
                ModeIcon(mode: mode, isSelected: modeManager.currentMode == mode) {
                    modeManager.changeMode(to: mode)  // ← 修正
                }
            }
        }
        .padding()
    }
}
