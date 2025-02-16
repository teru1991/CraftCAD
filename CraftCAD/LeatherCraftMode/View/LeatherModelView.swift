import SwiftUI
import RealityKit

struct LeatherModelView: View {
    @State private var leatherParts: [LeatherPart] = []
    @State private var selectedPart: LeatherPart?
    private let partManager = LeatherPartManager()
    private let seamManager = SeamManager()
    private let interactionManager = InteractionManager()
    private let cameraManager = CameraManager()
    private let materialManager = MaterialManager()

    var body: some View {
        VStack {
            RealityView { content in
                let anchor = AnchorEntity(world: .zero)
                for part in leatherParts {
                    let entity = ModelEntity(mesh: .generateBox(size: [0.1, 0.1, 0.002]))
                    entity.model?.materials = [materialManager.createLeatherMaterial()]
                    
                    if let selected = selectedPart, selected.id == part.id {
                        entity.model?.materials = [SimpleMaterial(color: .yellow, isMetallic: false)] // ハイライト
                    }
                    
                    if seamManager.showSeams {
                        entity.model?.materials.append(SimpleMaterial(color: .red, isMetallic: false)) // シーム強調
                    }
                    
                    anchor.addChild(entity)
                }
                content.add(anchor)
            }
            .frame(height: 400)

            HStack {
                Button("パーツ追加", action: addLeatherPart)
                Button("パーツ削除", action: removeLeatherPart)
                Button("パーツ分割", action: splitPart)
                Button("パーツ結合", action: mergePart)
                Button("シーム表示切替", action: toggleSeams)
            }
            .padding()

            HStack {
                Button("移動", action: movePart)
                Button("回転", action: rotatePart)
                Button("スナップ", action: snapPart)
                Button("ズームイン", action: zoomIn)
                Button("ズームアウト", action: zoomOut)
            }
        }
    }

    private func addLeatherPart() {
        let newPart = LeatherPart(id: UUID(), vertices: [], edges: [], faces: [])
        leatherParts.append(newPart)
    }

    private func removeLeatherPart() {
        if let selected = selectedPart {
            leatherParts.removeAll { $0.id == selected.id }
            selectedPart = nil
        }
    }

    private func splitPart() {
        guard let selected = selectedPart, let index = selected.edges.indices.randomElement(),
              let (part1, part2) = partManager.splitPart(selected, at: index) else { return }
        leatherParts.append(contentsOf: [part1, part2])
    }

    private func mergePart() {
        guard leatherParts.count > 1 else { return }
        var part1 = leatherParts.removeFirst()
        let part2 = leatherParts.removeFirst()
        partManager.mergeParts(&part1, part2)
        leatherParts.append(part1)
    }

    private func toggleSeams() {
        seamManager.toggleSeamVisibility()
    }

    private func movePart() {
        if let selectedIndex = leatherParts.firstIndex(where: { $0.id == selectedPart?.id }) {
            interactionManager.movePart(&leatherParts[selectedIndex], by: SIMD3<Float>(0.1, 0.0, 0.0))
        }
    }

    private func rotatePart() {
        if let selectedIndex = leatherParts.firstIndex(where: { $0.id == selectedPart?.id }) {
            interactionManager.rotatePart(&leatherParts[selectedIndex], angleY: .pi / 4) // ✅ Y軸回転
        }
    }

    private func snapPart() {
        if let selectedIndex = leatherParts.firstIndex(where: { $0.id == selectedPart?.id }) {
            for i in leatherParts[selectedIndex].vertices.indices {
                leatherParts[selectedIndex].vertices[i].x = interactionManager.snapToGrid(leatherParts[selectedIndex].vertices[i].x, gridSize: 0.1)
                leatherParts[selectedIndex].vertices[i].y = interactionManager.snapToGrid(leatherParts[selectedIndex].vertices[i].y, gridSize: 0.1)
                leatherParts[selectedIndex].vertices[i].z = interactionManager.snapToGrid(leatherParts[selectedIndex].vertices[i].z, gridSize: 0.1)
            }
        }
    }

    private func zoomIn() {
        cameraManager.zoomIn()
    }

    private func zoomOut() {
        cameraManager.zoomOut()
    }
}

#Preview {
    LeatherModelView()
}
