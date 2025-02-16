//
//  StorageService.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import Foundation

class StorageService {
    static let shared = StorageService()
    
    func getStorageInfo() -> Storage {
        return Storage(usage: 30, total: 100)
    }
    
    func sync() {
        print("クラウド同期を開始")
    }
}
