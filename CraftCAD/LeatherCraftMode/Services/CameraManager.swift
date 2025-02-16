//
//  CameraManager.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/08.
//


import Foundation
import RealityKit

class CameraManager {
    var zoomLevel: Float = 1.0

    func zoomIn() {
        zoomLevel = max(zoomLevel - 0.1, 0.5)
    }

    func zoomOut() {
        zoomLevel = min(zoomLevel + 0.1, 2.0)
    }

    func rotateCamera(by angle: Float, around axis: SIMD3<Float>) -> Transform {
        return Transform(rotation: simd_quatf(angle: angle, axis: axis))
    }
}
