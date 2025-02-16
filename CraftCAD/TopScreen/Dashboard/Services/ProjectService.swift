//
//  ProjectService.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//

import Foundation

class ProjectService {
    static let shared = ProjectService()
    
    func getRecentProjects(tag: String? = nil) -> [Project] {
        let allProjects = ProjectManager.shared.fetchProjects()
        
        if let tag = tag {
            return allProjects.filter { TagManager.shared.getTags(for: $0.id).contains(tag) }
        }
        
        return allProjects
    }
}
