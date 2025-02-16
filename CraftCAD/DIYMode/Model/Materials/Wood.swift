import SceneKit

struct Wood {
    static func createMaterial() -> SCNMaterial {
        let material = SCNMaterial()
        material.diffuse.contents = UIImage(named: "wood_texture") ?? UIColor.brown
        material.specular.contents = UIColor.white
        material.roughness.contents = 0.7
        material.metalness.contents = 0.1
        material.normal.contents = UIImage(named: "wood_normal")
        material.isDoubleSided = true
        return material
    }
}
