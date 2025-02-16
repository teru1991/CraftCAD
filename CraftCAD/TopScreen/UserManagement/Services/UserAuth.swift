import Foundation
import FirebaseAuth

/// ユーザー認証管理
class UserAuth: ObservableObject {
    @Published var isAuthenticated: Bool = false
    // FirebaseAuth の User 型を利用する
    @Published var currentUser: FirebaseAuth.User?
    
    static let shared = UserAuth()
    
    private init() {
        Auth.auth().addStateDidChangeListener { _, user in
            self.currentUser = user
            self.isAuthenticated = (user != nil)
        }
    }
    
    /// Googleでログイン
    func signInWithGoogle() {
        // Googleサインインの実装（GoogleSignIn SDKが必要）
    }
    
    /// Appleでログイン
    func signInWithApple() {
        // Appleサインインの実装（Sign in with Apple SDKが必要）
    }
    
    /// ログアウト
    func signOut() {
        do {
            try Auth.auth().signOut()
            self.isAuthenticated = false
            self.currentUser = nil
        } catch {
            print("ログアウトに失敗しました: \(error.localizedDescription)")
        }
    }
}
