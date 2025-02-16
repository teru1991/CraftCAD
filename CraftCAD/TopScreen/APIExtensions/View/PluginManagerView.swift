//
//  PluginManagerView.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import SwiftUI

struct PluginManagerView: View {
    @State private var pluginList: [String] = PluginManager().listInstalledPlugins()
    @State private var newPluginPath: String = ""

    var body: some View {
        VStack {
            Text("プラグイン管理")
                .font(.title)
                .padding()

            List(pluginList, id: \.self) { plugin in
                HStack {
                    Text(plugin)
                    Spacer()
                    Button("削除") {
                        if PluginManager().removePlugin(named: plugin) {
                            pluginList = PluginManager().listInstalledPlugins()
                        }
                    }
                }
            }

            HStack {
                TextField("プラグインのパス", text: $newPluginPath)
                    .textFieldStyle(RoundedBorderTextFieldStyle())
                Button("インストール") {
                    if PluginManager().installPlugin(from: newPluginPath) {
                        pluginList = PluginManager().listInstalledPlugins()
                    }
                }
            }
            .padding()
        }
    }
}
