import Foundation
import SceneKit
import CoreGraphics
import UIKit

class TwoDDesign: ObservableObject { 
    @Published var parts: [TwoDPart]

    init() {
        self.parts = []
    }

    // ✅ 3Dモデルを2D型紙へ展開
    func flattenTo2D(from shape: SCNNode) {
        guard let geometry = shape.geometry else { return }
        
        let boundingBox = geometry.boundingBox
        let width = CGFloat(boundingBox.max.x - boundingBox.min.x) // Float → CGFloat
        let height = CGFloat(boundingBox.max.y - boundingBox.min.y) // Float → CGFloat

        let newPart = TwoDPart(width: width, height: height)
        parts.append(newPart)
    }

    // ✅ 縫い代を追加
    func addSeamAllowance(width: CGFloat) {
        for part in parts {
            part.width += width * 2
            part.height += width * 2
        }
    }

    // ✅ 型紙の最適配置（ネスティング）
    func optimizePatternLayout() {
        parts.sort { $0.width * $0.height > $1.width * $1.height }
        
        var xOffset: CGFloat = 0
        var yOffset: CGFloat = 0
        let maxWidth: CGFloat = 500 // 仮の材料幅

        for part in parts {
            if xOffset + part.width > maxWidth {
                xOffset = 0
                yOffset += part.height + 10
            }
            part.position = CGPoint(x: xOffset, y: yOffset)
            xOffset += part.width + 10
        }
    }

    // ✅ 型紙をPDFでエクスポート
    func exportToPDF(fileName: String) {
        let pdfPath = FileManager.default.temporaryDirectory.appendingPathComponent("\(fileName).pdf")

        let pdfRenderer = UIGraphicsPDFRenderer(bounds: CGRect(x: 0, y: 0, width: 600, height: 800))
        do {
            try pdfRenderer.writePDF(to: pdfPath, withActions: { context in
                context.beginPage()
                
                for part in parts {
                    let rect = CGRect(x: part.position.x, y: part.position.y, width: part.width, height: part.height)
                    context.cgContext.setStrokeColor(UIColor.black.cgColor)
                    context.cgContext.stroke(rect)
                }
            })
            print("PDF exported to \(pdfPath)")
        } catch {
            print("Failed to create PDF: \(error)")
        }
    }
}

// ✅ 2D型紙のパーツクラス
class TwoDPart {
    var width: CGFloat
    var height: CGFloat
    var position: CGPoint

    init(width: CGFloat, height: CGFloat) {
        self.width = width
        self.height = height
        self.position = .zero
    }
}
