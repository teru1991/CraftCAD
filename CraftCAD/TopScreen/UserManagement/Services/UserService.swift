import Foundation

class UserService {
    static let shared = UserService()
    
    func getUserInfo() -> User {
        return User(id: "123", name: "板橋慶治", role: .admin) // ✅ 修正: `role` を追加
    }
}
