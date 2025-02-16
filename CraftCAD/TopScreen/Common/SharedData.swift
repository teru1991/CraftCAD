//
//  SharedData.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import Foundation

class SharedData: ObservableObject {
    @Published var currentProject: String = ""
    @Published var designData: [String: Any] = [:]  // 各モードの設計データを保存
}
