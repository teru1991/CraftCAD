import Foundation

struct Project: Identifiable {
    let id: String
    var name: String  // ✅ `let` → `var` に変更
    var lastModified: Date // ✅ `let` → `var` に変更
}
