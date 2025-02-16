import SceneKit

struct Plastic {
    static func createMaterial() -> SCNMaterial {
        let material = SCNMaterial()
        material.diffuse.contents = UIColor.white
        material.specular.contents = UIColor.lightGray
        material.roughness.contents = 0.5
        material.metalness.contents = 0.0
        material.normal.contents = nil // プラスチックは基本的に凹凸が少ないため
        material.isDoubleSided = true
        return material
    }
}
