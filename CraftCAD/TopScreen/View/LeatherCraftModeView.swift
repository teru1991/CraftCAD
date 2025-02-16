//
//  LeatherCraftModeView.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


// CraftCAD/TopScreen/View/LeatherCraftModeView.swift
import SwiftUI

struct LeatherCraftModeView: View {
    @EnvironmentObject var sharedData: SharedData
    
    var body: some View {
        VStack {
            Text("レザークラフトモード")
                .font(.largeTitle)
                .padding()
            
            Text("現在のプロジェクト: \(sharedData.currentProject)")
            
            Button("DIYデータを取得") {
                if let diyData = sharedData.designData["DIY"] as? String {
                    print("DIYデータ: \(diyData)")
                }
            }
            
            Spacer()
        }
        .background(Color.brown.opacity(0.1))
    }
}

