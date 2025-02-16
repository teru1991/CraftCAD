import Foundation
import SceneKit

enum ShapeType {
    case box, cylinder, sphere, cone, torus, capsule, pyramid
}

class Shape {
    var id: UUID
    var type: ShapeType
    var node: SCNNode
    
    init(type: ShapeType) {
        self.id = UUID()
        self.type = type
        self.node = Shape.createNode(for: type)
    }
    
    static func createNode(for type: ShapeType) -> SCNNode {
        switch type {
        case .box:
            let geometry = SCNBox(width: 1.0, height: 1.0, length: 1.0, chamferRadius: 0.1)
            geometry.firstMaterial?.diffuse.contents = UIColor.brown
            return SCNNode(geometry: geometry)
            
        case .cylinder:
            let geometry = SCNCylinder(radius: 0.5, height: 1.0)
            geometry.firstMaterial?.diffuse.contents = UIColor.gray
            return SCNNode(geometry: geometry)
        
        case .sphere:
            let geometry = SCNSphere(radius: 0.5)
            geometry.firstMaterial?.diffuse.contents = UIColor.red
            return SCNNode(geometry: geometry)
            
        case .cone:
            let geometry = SCNCone(topRadius: 0.0, bottomRadius: 0.5, height: 1.0)
            geometry.firstMaterial?.diffuse.contents = UIColor.orange
            return SCNNode(geometry: geometry)
            
        case .torus:
            let geometry = SCNTorus(ringRadius: 0.6, pipeRadius: 0.2)
            geometry.firstMaterial?.diffuse.contents = UIColor.purple
            return SCNNode(geometry: geometry)
        
        case .capsule:
            let geometry = SCNCapsule(capRadius: 0.3, height: 1.2)
            geometry.firstMaterial?.diffuse.contents = UIColor.blue
            return SCNNode(geometry: geometry)
        
        case .pyramid:
            let geometry = SCNPyramid(width: 1.0, height: 1.0, length: 1.0)
            geometry.firstMaterial?.diffuse.contents = UIColor.green
            return SCNNode(geometry: geometry)
        }
    }
}
