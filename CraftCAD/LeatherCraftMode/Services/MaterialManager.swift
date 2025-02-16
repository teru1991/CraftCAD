import Foundation
import RealityKit
import UIKit // ✅ 追加

class MaterialManager {
    func createLeatherMaterial() -> SimpleMaterial {
        let baseColor = UIColor.brown // ✅ これで認識される
        let material = SimpleMaterial(color: baseColor, isMetallic: false)
        return material
    }
}
