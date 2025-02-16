//
//  SettingsManager.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/16.
//


import Foundation
import SwiftUI

/// アプリの設定を管理するクラス
class SettingsManager: ObservableObject {
    static let shared = SettingsManager() // シングルトンインスタンス

    @AppStorage("appLanguage") var appLanguage: String = "ja" // 言語設定（デフォルト: 日本語）
    @AppStorage("isDarkMode") var isDarkMode: Bool = false // ダークモード設定
    @AppStorage("shortcutEnabled") var shortcutEnabled: Bool = true // ショートカットキーの有効化

    @Published var currentTheme: Theme = .light

    init() {
        loadUserSettings()
    }

    /// 設定をロード
    func loadUserSettings() {
        print("🔄 ユーザー設定をロード中...")
        currentTheme = isDarkMode ? .dark : .light
    }

    /// 言語を変更
    func updateLanguage(to language: String) {
        appLanguage = language
        print("🌐 言語を \(language) に変更しました")
    }

    /// ダークモードの切り替え
    func toggleDarkMode() {
        isDarkMode.toggle()
        currentTheme = isDarkMode ? .dark : .light
        print("🌙 ダークモード: \(isDarkMode ? "ON" : "OFF")")
    }

    /// ショートカットキーの有効/無効を切り替え
    func toggleShortcut() {
        shortcutEnabled.toggle()
        print("⌨️ ショートカットキー: \(shortcutEnabled ? "有効" : "無効")")
    }
}

/// アプリのテーマ（ダーク/ライト）を定義
enum Theme {
    case light, dark
}
