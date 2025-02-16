import Foundation

class PluginManager {
    private var installedPlugins: [String: URL] = [:]

    func installPlugin(from filePath: String) -> Bool {
        let fileURL = URL(fileURLWithPath: filePath)
        let pluginName = fileURL.deletingPathExtension().lastPathComponent

        if installedPlugins[pluginName] != nil {
            print("Plugin \(pluginName) is already installed.")
            return false
        }

        installedPlugins[pluginName] = fileURL
        print("Plugin \(pluginName) installed successfully from \(filePath)")
        return true
    }

    func removePlugin(named pluginName: String) -> Bool {
        guard let _ = installedPlugins.removeValue(forKey: pluginName) else {
            print("Plugin \(pluginName) not found.")
            return false
        }

        print("Plugin \(pluginName) removed successfully.")
        return true
    }

    func listInstalledPlugins() -> [String] {
        return Array(installedPlugins.keys)
    }
}
