import Foundation

/// ユーザーの役割
enum UserRole: String, Codable, CaseIterable {
    case admin = "Admin"
    case user = "User"
}

/// ユーザー管理モデル
class User: Identifiable, ObservableObject {
    let id: String
    @Published var name: String
    @Published var role: UserRole

    init(id: String, name: String, role: UserRole) {
        self.id = id
        self.name = name
        self.role = role
    }

    /// ユーザー情報を更新
    func update(name: String? = nil, role: UserRole? = nil) {
        if let newName = name {
            self.name = newName
        }
        if let newRole = role {
            self.role = newRole
        }
    }
}
