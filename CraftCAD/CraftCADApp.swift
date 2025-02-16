import SwiftUI

@main
struct CraftCADApp: App {
    @StateObject private var modeManager = ModeManager()
    @StateObject private var sharedData = SharedData()
    @StateObject private var settingsManager = SettingsManager()

    var body: some Scene {
        WindowGroup {
            NavigationView {
                TopScreenView() // ✅ クラウド関連の処理は `TopScreenView` で実装
                    .environmentObject(modeManager)
                    .environmentObject(sharedData)
                    .environmentObject(settingsManager) // ✅ 設定管理は保持
                    .onAppear {
                        setupApp()
                    }
            }
        }
    }
    
    /// アプリ起動時のセットアップ
    private func setupApp() {
        settingsManager.loadUserSettings() // ✅ 設定情報のロードのみ
    }
}
