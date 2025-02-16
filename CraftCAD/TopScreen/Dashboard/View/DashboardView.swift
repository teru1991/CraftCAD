//
//  DashboardView.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//

import SwiftUI

struct DashboardView: View {
    var body: some View {
        NavigationView {
            VStack {
                UserInfoPanel()
                ProjectListView()  // 変更点：独立した ProjectListView を利用
                StorageStatus()
            }
            .navigationTitle("Dashboard")
            .padding()
        }
    }
}
