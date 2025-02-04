import SceneKit
import SwiftUI

class ModelViewModel: ObservableObject {
    @Published var scene: SCNScene
    private var rootNode: SCNNode
    @Published var selectedNode: SCNNode?
    
    @Published var selectedSize: CGFloat = 1.0
    @Published var rotationX: CGFloat = 0.0
    @Published var rotationY: CGFloat = 0.0
    @Published var rotationZ: CGFloat = 0.0
    
    init() {
        let scene = SCNScene()
        self.scene = scene
        self.rootNode = scene.rootNode
        setupScene()
    }
    
    private func setupScene() {
        let cameraNode = SCNNode()
        cameraNode.camera = SCNCamera()
        cameraNode.position = SCNVector3(0, 3, 5)
        self.rootNode.addChildNode(cameraNode)
        
        let lightNode = SCNNode()
        lightNode.light = SCNLight()
        lightNode.light?.type = .omni
        lightNode.position = SCNVector3(0, 5, 5)
        self.rootNode.addChildNode(lightNode)
        
        let floor = SCNFloor()
        floor.firstMaterial?.diffuse.contents = UIColor.lightGray
        let floorNode = SCNNode(geometry: floor)
        self.rootNode.addChildNode(floorNode)
    }
    
    func addShape(_ type: ShapeType, at position: SCNVector3) {
        let geometry: SCNGeometry
        
        switch type {
        case .box:
            geometry = SCNBox(width: 1.0, height: 1.0, length: 1.0, chamferRadius: 0.1)
        case .cylinder:
            geometry = SCNCylinder(radius: 0.5, height: 1.0)
        case .sphere:
            geometry = SCNSphere(radius: 0.5)
        }
        
        geometry.firstMaterial?.diffuse.contents = UIColor.gray
        
        let node = SCNNode(geometry: geometry)
        node.position = snapToGrid(position)
        node.name = UUID().uuidString
        
        self.rootNode.addChildNode(node)
    }
    
    func selectNode(_ node: SCNNode) {
        self.selectedNode = node
    }
    
    func moveNode(_ node: SCNNode, to position: SCNVector3) {
        node.position = snapToGrid(position)
    }
    
    func duplicateSelectedNode() {
        guard let selectedNode = selectedNode else { return }
        let duplicatedNode = selectedNode.clone()
        duplicatedNode.position = SCNVector3(selectedNode.position.x + 1, selectedNode.position.y, selectedNode.position.z)
        rootNode.addChildNode(duplicatedNode)
    }
    
    func deleteSelectedNode() {
        guard let selectedNode = selectedNode else { return }
        selectedNode.removeFromParentNode()
        self.selectedNode = nil
    }
    
    func applyMaterial(color: UIColor? = nil, texture: UIImage? = nil) {
        guard let selectedNode = selectedNode, let geometry = selectedNode.geometry else { return }
        
        if let color = color {
            geometry.firstMaterial?.diffuse.contents = color
        }
        if let texture = texture {
            geometry.firstMaterial?.diffuse.contents = texture
        }
    }
    
    func snapToGrid(_ position: SCNVector3, gridSize: Float = 1.0) -> SCNVector3 {
        let snappedX = round(position.x / gridSize) * gridSize
        let snappedY = round(position.y / gridSize) * gridSize
        let snappedZ = round(position.z / gridSize) * gridSize
        return SCNVector3(snappedX, snappedY, snappedZ)
    }
}
