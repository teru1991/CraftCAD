//
//  LeatherMaterial.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


// レザーの素材設定
class LeatherMaterial {
    static func createMaterial() -> SCNMaterial {
        let material = SCNMaterial()
        material.diffuse.contents = UIImage(named: "leather_texture")
        material.specular.contents = UIColor.darkGray
        material.roughness.contents = 0.8
        material.metalness.contents = 0.0
        material.normal.contents = UIImage(named: "leather_normal") // 法線マップ追加
        return material
    }
}