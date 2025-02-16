//
//  TopScreenView.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/07.
//


import SwiftUI

/// トップスクリーンビュー
struct TopScreenView: View {
    @StateObject private var modeManager = ModeManager()
    @StateObject private var sharedData = SharedData() // 共有データを管理

    var body: some View {
        VStack {
            ModeSelectorView()
                .environmentObject(modeManager)
                .environmentObject(sharedData) // 共有データを各モードに渡す
                .padding()

            // ✅ モードに応じたビューを表示
            switch modeManager.currentMode {
            case .diy:
                DIYModeView().environmentObject(sharedData)
            case .leatherCraft:
                LeatherCraftModeView().environmentObject(sharedData)
            case .pro:
                ProModeView().environmentObject(sharedData)
            }
        }
        .navigationTitle("CraftCAD")
    }
}
