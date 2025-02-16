//
//  TagManager.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


//
//  TagManager.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//

import Foundation

class TagManager {
    static let shared = TagManager()
    
    private var projectTags: [String: [String]] = [:] // プロジェクトIDごとのタグリスト
    
    // タグを追加
    func addTag(to projectID: String, tag: String) {
        if projectTags[projectID] == nil {
            projectTags[projectID] = []
        }
        projectTags[projectID]?.append(tag)
    }
    
    // タグを削除
    func removeTag(from projectID: String, tag: String) {
        projectTags[projectID]?.removeAll { $0 == tag }
    }
    
    // プロジェクトごとのタグを取得
    func getTags(for projectID: String) -> [String] {
        return projectTags[projectID] ?? []
    }
}
