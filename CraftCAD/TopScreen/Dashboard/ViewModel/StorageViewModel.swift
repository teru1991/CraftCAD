//
//  StorageViewModel.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import SwiftUI
import Combine

class StorageViewModel: ObservableObject {
    @Published var usage: Double = 0
    @Published var total: Double = 100
    
    func fetchStorageStatus() {
        let status = StorageService.shared.getStorageInfo()
        self.usage = status.usage
        self.total = status.total
    }
    
    func syncStorage() {
        StorageService.shared.sync()
    }
}

