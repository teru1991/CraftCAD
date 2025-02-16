import SwiftUI

class DashboardViewModel: ObservableObject {
    @Published var currentUser: User? = nil
    @Published var recentProjects: [Project] = []
    @Published var localStorage: String = "0 GB used"
    @Published var cloudStorage: String = "0 GB used"
    
    func loadDashboardData() {
        // ✅ 修正: `role` に `.admin` を使用（UserRole 型）
        self.currentUser = User(id: UUID().uuidString, name: "John Doe", role: .admin)

        // Simulate loading projects
        self.recentProjects = [
            Project(id: UUID().uuidString, name: "Project Alpha", lastModified: Date()),
            Project(id: UUID().uuidString, name: "Project Beta", lastModified: Date()),
            Project(id: UUID().uuidString, name: "Project Gamma", lastModified: Date())
        ]
        
        // Simulate loading storage data
        self.localStorage = "20 GB used"
        self.cloudStorage = "15 GB used"
    }
}
