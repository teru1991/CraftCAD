import Foundation

class UserViewModel: ObservableObject {
    @Published var users: [User] = []
    
    /// ユーザーを取得
    func fetchUsers() {
        self.users = [
            User(id: UUID().uuidString, name: "田中 太郎", role: .admin),
            User(id: UUID().uuidString, name: "佐藤 花子", role: .user)
        ]
    }
    
    /// ユーザーを追加
    func addUser(id: String, name: String, role: UserRole) {
        let newUser = User(id: id, name: name, role: role)
        users.append(newUser)
    }
    
    /// ユーザーを削除
    func removeUser(byId id: String) {
        users.removeAll { $0.id == id }
    }
    
    /// ユーザー情報を更新
    func updateUser(id: String, name: String?, role: UserRole?) {
        if let user = users.first(where: { $0.id == id }) {
            user.update(name: name, role: role)
        }
    }
}
