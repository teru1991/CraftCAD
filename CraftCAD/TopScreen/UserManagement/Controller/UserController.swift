import Foundation
import SwiftUI
/// ユーザーコントローラー
class UserController {
    static let shared = UserController()
    private let userDefaultsKey = "UserPreferredMode"
    
    private init() {}
    
    func saveUserMode(_ mode: AppMode) {
        UserDefaults.standard.set(mode.rawValue, forKey: userDefaultsKey)
    }
    
    func loadUserMode() -> AppMode {
        if let savedMode = UserDefaults.standard.string(forKey: userDefaultsKey),
           let mode = AppMode(rawValue: savedMode) {
            return mode
        }
        return .diy
    }
}
