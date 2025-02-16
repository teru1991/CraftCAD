import Foundation
import simd

class InteractionManager {
    
    /// パーツを指定のオフセットで移動
    func movePart(_ part: inout LeatherPart, by offset: SIMD3<Float>) {
        for i in part.vertices.indices {
            part.vertices[i] += offset
        }
    }

    /// 指定の角度でパーツを回転（X, Y, Z軸対応）
    func rotatePart(_ part: inout LeatherPart, angleX: Float = 0, angleY: Float = 0, angleZ: Float = 0) {
        let rotationMatrix = float4x4.rotationX(angleX) *
                             float4x4.rotationY(angleY) *
                             float4x4.rotationZ(angleZ)

        for i in part.vertices.indices {
            let position = SIMD4<Float>(part.vertices[i], 1) // 4Dベクトルに拡張
            let rotatedPosition = rotationMatrix * position  // 変換行列を適用
            part.vertices[i] = SIMD3<Float>(rotatedPosition.x, rotatedPosition.y, rotatedPosition.z) // ✅ 修正！
        }
    }

    /// グリッドスナップ処理（位置を指定のグリッドサイズに揃える）
    func snapToGrid(_ value: Float, gridSize: Float) -> Float {
        return round(value / gridSize) * gridSize
    }
}

// MARK: - float4x4 拡張（回転行列）
extension float4x4 {
    /// X軸回転行列を生成
    static func rotationX(_ angle: Float) -> float4x4 {
        let cosA = cos(angle)
        let sinA = sin(angle)
        return float4x4(
            SIMD4<Float>(1, 0, 0, 0),
            SIMD4<Float>(0, cosA, -sinA, 0),
            SIMD4<Float>(0, sinA, cosA, 0),
            SIMD4<Float>(0, 0, 0, 1)
        )
    }
    
    /// Y軸回転行列を生成
    static func rotationY(_ angle: Float) -> float4x4 {
        let cosA = cos(angle)
        let sinA = sin(angle)
        return float4x4(
            SIMD4<Float>(cosA, 0, sinA, 0),
            SIMD4<Float>(0, 1, 0, 0),
            SIMD4<Float>(-sinA, 0, cosA, 0),
            SIMD4<Float>(0, 0, 0, 1)
        )
    }
    
    /// Z軸回転行列を生成
    static func rotationZ(_ angle: Float) -> float4x4 {
        let cosA = cos(angle)
        let sinA = sin(angle)
        return float4x4(
            SIMD4<Float>(cosA, -sinA, 0, 0),
            SIMD4<Float>(sinA, cosA, 0, 0),
            SIMD4<Float>(0, 0, 1, 0),
            SIMD4<Float>(0, 0, 0, 1)
        )
    }
}
