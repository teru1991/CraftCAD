//
//  ThemeManager.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import SwiftUI

class ThemeManager {
    static let shared = ThemeManager()
    
    func applyTheme(isDarkMode: Bool) {
        DispatchQueue.main.async {
            UIApplication.shared.windows.first?.overrideUserInterfaceStyle = isDarkMode ? .dark : .light
        }
    }
}
