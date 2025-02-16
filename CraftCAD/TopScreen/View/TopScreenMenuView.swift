//
//  MainMenuView.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import SwiftUI

struct TopScreenMenuView: View {
    var body: some View {
        NavigationView {
            List {
                NavigationLink("🔌 プラグイン管理", destination: PluginManagerView())
                NavigationLink("📜 スクリプト実行", destination: ScriptExecutorView())
                NavigationLink("📂 CADファイル管理", destination: CADFileManagerView())
                NavigationLink("⚙️ 設定", destination: SettingsView())
            }
            .navigationTitle("メニュー")
        }
    }
}
