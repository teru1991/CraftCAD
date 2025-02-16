import Foundation
import simd

struct LeatherPart: Identifiable {
    let id: UUID
    var vertices: [SIMD3<Float>] // 頂点リスト
    var edges: [(Int, Int)] // エッジ（頂点のペア）
    var faces: [[Int]] // フェース（面）

    var thickness: Float = 2.0  // レザーの厚み
    var curvature: Float = 0.0  // 曲率

    /// `mutating` を削除し、新しい `LeatherPart` を返すメソッドに変更
    func split(at edgeIndex: Int) -> (LeatherPart, LeatherPart)? {
        guard edgeIndex < edges.count else { return nil }
        let edge = edges[edgeIndex]

        let part1 = LeatherPart(id: UUID(), vertices: [vertices[edge.0]], edges: [], faces: [])
        let part2 = LeatherPart(id: UUID(), vertices: [vertices[edge.1]], edges: [], faces: [])

        return (part1, part2)
    }

    mutating func merge(with other: LeatherPart) {
        self.vertices.append(contentsOf: other.vertices)
        self.edges.append(contentsOf: other.edges)
        self.faces.append(contentsOf: other.faces)
    }
}
