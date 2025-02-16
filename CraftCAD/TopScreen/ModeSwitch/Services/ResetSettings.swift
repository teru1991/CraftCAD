//
//  ResetSettings.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import Foundation

class ResetSettings {
    static func resetAll() {
        let defaults = UserDefaults.standard
        defaults.removeObject(forKey: "AppLanguage")
        defaults.removeObject(forKey: "DarkMode")
        defaults.removeObject(forKey: "CustomShortcut")
        defaults.removeObject(forKey: "EnableNotifications")
    }
}
