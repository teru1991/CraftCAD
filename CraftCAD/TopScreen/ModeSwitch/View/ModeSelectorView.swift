import Foundation
import SwiftUI


/// モード選択UI
struct ModeSelectorView: View {
    @EnvironmentObject var modeManager: ModeManager
    
    var body: some View {
        VStack {
            Text("Select Mode")
                .font(.headline)
            Picker("Mode", selection: $modeManager.currentMode) {
                ForEach(AppMode.allCases, id: \.self) { mode in
                    Text(mode.rawValue).tag(mode)
                }
            }
            .pickerStyle(SegmentedPickerStyle())
            .onChange(of: modeManager.currentMode) { newMode in
                modeManager.changeMode(to: newMode)
                UserController.shared.saveUserMode(newMode) // ユーザー設定に保存
            }
        }
        .padding()
    }
}
