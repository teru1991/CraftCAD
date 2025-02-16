//
//  ProModeView.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


// CraftCAD/TopScreen/View/ProModeView.swift
import SwiftUI

struct ProModeView: View {
    var body: some View {
        VStack {
            Text("プロモード")
                .font(.largeTitle)
                .padding()
            
            Spacer()
            
            Text("Fusion 360 や AutoCAD 相当の高度な3D/2D CAD機能を搭載。")
                .font(.body)
            
            Spacer()
        }
        .background(Color.blue.opacity(0.1))
    }
}
