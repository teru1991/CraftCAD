import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var modeManager: ModeManager
    @State private var isAutoSyncEnabled: Bool = false
    @State private var selectedLanguage: String = UserDefaults.standard.string(forKey: "AppLanguage") ?? "English"
    @State private var isDarkMode: Bool = UserDefaults.standard.bool(forKey: "DarkMode")

    // 🔹 新規追加（設定管理）
    @State private var pluginDirectory: String = UserDefaults.standard.string(forKey: "PluginDirectory") ?? "/plugins"
    @State private var scriptDirectory: String = UserDefaults.standard.string(forKey: "ScriptDirectory") ?? "/scripts"
    @State private var cadSaveDirectory: String = UserDefaults.standard.string(forKey: "CADSaveDirectory") ?? "/cad_files"
    @State private var errorLogs: [String] = LogManager.shared.getLogs()

    var body: some View {
        NavigationView {
            Form {
                // 🔹 モード設定
                Section(header: Text("Mode Settings")) {
                    Picker("Mode", selection: $modeManager.currentMode) {
                        ForEach(ModeManager.Mode.allCases, id: \.self) { mode in
                            Text(mode.rawValue).tag(mode)
                        }
                    }
                    .pickerStyle(MenuPickerStyle())
                    .onChange(of: modeManager.currentMode) { newMode in
                        modeManager.switchMode(to: newMode)
                    }
                }

                // 🔹 言語設定
                Section(header: Text("Language Settings")) {
                    Picker("Language", selection: $selectedLanguage) {
                        Text("English").tag("English")
                        Text("日本語").tag("Japanese")
                    }
                    .pickerStyle(SegmentedPickerStyle())
                    .onChange(of: selectedLanguage) { newLanguage in
                        UserDefaults.standard.set(newLanguage, forKey: "AppLanguage")
                    }
                }

                // 🔹 UIテーマ設定
                Section(header: Text("Theme Settings")) {
                    Toggle("Dark Mode", isOn: $isDarkMode)
                        .onChange(of: isDarkMode) { value in
                            UserDefaults.standard.set(value, forKey: "DarkMode")
                            ThemeManager.shared.applyTheme(isDarkMode: value)
                        }
                }

                // 🔹 ショートカット設定
                Section(header: Text("Shortcut Settings")) {
                    NavigationLink(destination: ShortcutSettingsView()) {
                        Text("Customize Shortcuts")
                    }
                }

                // 🔹 プラグイン管理
                Section(header: Text("Plugin Settings")) {
                    TextField("Plugin Directory", text: $pluginDirectory)
                        .onChange(of: pluginDirectory) { newValue in
                            UserDefaults.standard.set(newValue, forKey: "PluginDirectory")
                        }
                }

                // 🔹 スクリプト管理
                Section(header: Text("Script Execution Settings")) {
                    TextField("Script Directory", text: $scriptDirectory)
                        .onChange(of: scriptDirectory) { newValue in
                            UserDefaults.standard.set(newValue, forKey: "ScriptDirectory")
                        }
                }

                // 🔹 CADデータ管理
                Section(header: Text("CAD File Settings")) {
                    TextField("CAD Save Directory", text: $cadSaveDirectory)
                        .onChange(of: cadSaveDirectory) { newValue in
                            UserDefaults.standard.set(newValue, forKey: "CADSaveDirectory")
                        }
                }

                // 🔹 ログ表示
                Section(header: Text("Error Logs")) {
                    List(errorLogs, id: \.self) { log in
                        Text(log)
                    }
                }

                // 🔹 設定リセット
                Section {
                    Button("Reset to Default") {
                        ResetSettings.resetAll()
                    }
                    .foregroundColor(.red)
                }
            }
            .navigationTitle("Settings")
        }
    }
}
