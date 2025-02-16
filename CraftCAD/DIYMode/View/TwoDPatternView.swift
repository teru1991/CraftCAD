//
//  TwoDPatternView.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import SwiftUI

struct TwoDPatternView: View {
    var twoDDesign: TwoDDesign

    var body: some View {
        GeometryReader { geometry in
            Canvas { context, size in
                // twoDDesign.parts 配列の各パーツについて描画する
                for part in twoDDesign.parts {
                    let rect = CGRect(x: part.position.x, y: part.position.y, width: part.width, height: part.height)
                    context.stroke(Path(rect), with: .color(.black), lineWidth: 2)
                }
            }
        }
    }
}
