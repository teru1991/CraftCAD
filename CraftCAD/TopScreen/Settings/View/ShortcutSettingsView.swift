//
//  ShortcutSettingsView.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import SwiftUI

struct ShortcutSettingsView: View {
    @State private var customShortcut: String = UserDefaults.standard.string(forKey: "CustomShortcut") ?? "Ctrl + S"
    
    var body: some View {
        Form {
            TextField("Shortcut Key", text: $customShortcut)
                .onChange(of: customShortcut) { newValue in
                    UserDefaults.standard.set(newValue, forKey: "CustomShortcut")
                }
            
            Button("Reset to Default") {
                customShortcut = "Ctrl + S"
                UserDefaults.standard.set(customShortcut, forKey: "CustomShortcut")
            }
        }
        .navigationTitle("Shortcut Settings")
    }
}
