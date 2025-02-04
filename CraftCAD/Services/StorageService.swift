import Foundation
import SceneKit

class StorageService {
    static func saveScene(scene: SCNScene, fileName: String) {
        var objects: [[String: Any]] = []
        
        for node in scene.rootNode.childNodes {
            let data: [String: Any] = [
                "name": node.name ?? "Unknown",
                "x": node.position.x,
                "y": node.position.y,
                "z": node.position.z,
                "scale": node.scale.x
            ]
            objects.append(data)
        }

        let json = try? JSONSerialization.data(withJSONObject: objects, options: .prettyPrinted)
        let fileURL = FileManager.default.temporaryDirectory.appendingPathComponent("\(fileName).json")
        try? json?.write(to: fileURL)
    }
    
    static func loadScene(scene: SCNScene, fileName: String) {
        let fileURL = FileManager.default.temporaryDirectory.appendingPathComponent("\(fileName).json")
        guard let data = try? Data(contentsOf: fileURL),
              let objects = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] else { return }
        
        for obj in objects {
            let node = SCNNode()
            node.position = SCNVector3(
                obj["x"] as? Float ?? 0,
                obj["y"] as? Float ?? 0,
                obj["z"] as? Float ?? 0
            )
            node.scale = SCNVector3(
                obj["scale"] as? Float ?? 1,
                obj["scale"] as? Float ?? 1,
                obj["scale"] as? Float ?? 1
            )
            scene.rootNode.addChildNode(node)
        }
    }
}
