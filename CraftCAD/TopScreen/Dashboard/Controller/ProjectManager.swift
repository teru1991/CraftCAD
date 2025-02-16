import Foundation
import Combine

class ProjectManager: ObservableObject {
    static let shared = ProjectManager()

    @Published var projects: [Project] = []

    private init() {}

    func fetchProjects() -> [Project] {
        return projects.sorted(by: { $0.lastModified > $1.lastModified })
    }

    func createProject(name: String) {
        let newProject = Project(id: UUID().uuidString, name: name, lastModified: Date())
        projects.append(newProject)
    }

    func deleteProject(id: String) {
        projects.removeAll { $0.id == id }
    }

    func updateProject(id: String, newName: String) {
        if let index = projects.firstIndex(where: { $0.id == id }) {
            projects[index].name = newName
            projects[index].lastModified = Date()
        }
    }
}
