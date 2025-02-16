//
//  LanguageSelector.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import Foundation

class LanguageSelector {
    static let shared = LanguageSelector()
    
    func getCurrentLanguage() -> String {
        return UserDefaults.standard.string(forKey: "AppLanguage") ?? "English"
    }
    
    func setLanguage(_ language: String) {
        UserDefaults.standard.set(language, forKey: "AppLanguage")
    }
}
