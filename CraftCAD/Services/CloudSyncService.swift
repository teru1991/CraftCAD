import Foundation
import FirebaseFirestore
import SceneKit

class CloudSyncService {
    private let db = Firestore.firestore()
    
    func saveScene(scene: SCNScene, projectID: String) {
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

        db.collection("projects").document(projectID).setData(["nodes": objects]) { error in
            if let error = error {
                print("Error saving scene: \(error)")
            } else {
                print("Scene saved successfully")
            }
        }
    }
    
    func loadScene(scene: SCNScene, projectID: String) {
        db.collection("projects").document(projectID).getDocument { snapshot, error in
            guard let data = snapshot?.data(), let objects = data["nodes"] as? [[String: Any]] else { return }
            
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
}
