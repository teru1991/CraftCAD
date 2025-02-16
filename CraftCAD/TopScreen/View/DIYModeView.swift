//
//  DIYModeView.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


// CraftCAD/TopScreen/View/DIYModeView.swift
import SwiftUI

struct DIYModeView: View {
    @EnvironmentObject var sharedData: SharedData
    
    var body: some View {
        VStack {
            Text("DIYモード")
                .font(.largeTitle)
                .padding()
            
            TextField("プロジェクト名を入力", text: $sharedData.currentProject)
                .textFieldStyle(RoundedBorderTextFieldStyle())
                .padding()
            
            Button("データを保存") {
                sharedData.designData["DIY"] = "DIYの設計データ"
            }
            
            Spacer()
        }
        .background(Color.green.opacity(0.1))
    }
}
