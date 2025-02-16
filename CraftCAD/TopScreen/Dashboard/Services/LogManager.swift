//
//  LogManager.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import Foundation

class LogManager {
    static let shared = LogManager()
    
    private var logs: [String] = []

    private init() {}

    func addLog(_ message: String) {
        let timestamp = DateFormatter.localizedString(from: Date(), dateStyle: .short, timeStyle: .medium)
        logs.append("[\(timestamp)] \(message)")
    }

    func getLogs() -> [String] {
        return logs
    }
}
