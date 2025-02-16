//
//  ProjectHistory.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


//
//  ProjectHistory.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//

import Foundation

class ProjectHistory {
    static let shared = ProjectHistory()
    
    private var deletedProjects: [Project] = []
    
    // プロジェクトを削除して履歴に保存
    func deleteProject(_ project: Project) {
        deletedProjects.append(project)
        ProjectManager.shared.deleteProject(id: project.id)
    }
    
    // 削除したプロジェクトを復元
    func restoreProject(id: String) {
        if let project = deletedProjects.first(where: { $0.id == id }) {
            ProjectManager.shared.createProject(name: project.name)
            deletedProjects.removeAll { $0.id == id }
        }
    }
    
    // 削除履歴を取得
    func getDeletedProjects() -> [Project] {
        return deletedProjects
    }
}
