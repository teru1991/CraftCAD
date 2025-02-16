import Foundation
import SwiftUI

/// モードの種類
enum AppMode: String, CaseIterable {
    case diy = "DIYMode"
    case leatherCraft = "LeatherCraftMode"
    case pro = "ProMode"
}


/// モード管理クラス
class ModeManager: ObservableObject {
    @Published var currentMode: AppMode = .diy  // デフォルトでDIYモード
    
    /// ユーザー設定の保存・読み込み用
    private let userDefaultsKey = "selectedAppMode"
    
    init() {
        loadMode()  // 起動時にモードを読み込む
    }
    
    /// モードを変更する
    func changeMode(to newMode: AppMode) {
        guard currentMode != newMode else { return } // すでに選択中なら変更しない
        currentMode = newMode
        saveMode()  // ユーザー設定に保存
        print("Mode changed to: \(newMode.rawValue)")
    }
    
    /// モードを保存
    private func saveMode() {
        UserDefaults.standard.set(currentMode.rawValue, forKey: userDefaultsKey)
    }
    
    /// モードを読み込む
    private func loadMode() {
        if let savedMode = UserDefaults.standard.string(forKey: userDefaultsKey),
           let mode = AppMode(rawValue: savedMode) {
            currentMode = mode
        }
    }
}
