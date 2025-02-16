//
//  NotificationSettings.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import Foundation

class NotificationSettings {
    static let shared = NotificationSettings()
    
    func enableNotifications() {
        UserDefaults.standard.set(true, forKey: "EnableNotifications")
    }
    
    func disableNotifications() {
        UserDefaults.standard.set(false, forKey: "EnableNotifications")
    }
    
    func isNotificationsEnabled() -> Bool {
        return UserDefaults.standard.bool(forKey: "EnableNotifications")
    }
}
