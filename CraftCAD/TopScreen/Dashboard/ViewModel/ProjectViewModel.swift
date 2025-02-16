//
//  ProjectViewModel.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import SwiftUI
import Combine

class ProjectViewModel: ObservableObject {
    @Published var projects: [Project] = []
    
    func fetchProjects() {
        self.projects = ProjectService.shared.getRecentProjects()
    }
    
    func openProject(_ project: Project) {
        print("Opening project: \(project.name)")
    }
}
