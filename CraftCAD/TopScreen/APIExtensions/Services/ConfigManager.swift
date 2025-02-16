//
//  ConfigManager.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import Foundation

class ConfigManager {
    static let shared = ConfigManager()

    private let userDefaults = UserDefaults.standard

    private init() {}

    func setValue(_ value: String, forKey key: String) {
        userDefaults.set(value, forKey: key)
    }

    func getValue(forKey key: String) -> String? {
        return userDefaults.string(forKey: key)
    }
}
