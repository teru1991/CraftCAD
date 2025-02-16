import Foundation


class CADIntegration {
    
    enum CADFormat: String, CaseIterable {
        case dxf = "DXF"
        case svg = "SVG"
        case stl = "STL"
    }

    static func importData(from path: String) -> Data? {
        return path.isEmpty ? nil : Data()
    }


    func exportAndUpload(to format: CADFormat, data: Data, completion: @escaping (Bool) -> Void) {
        let filePath = FileManager.default.temporaryDirectory.appendingPathComponent("exported_file.\(format.rawValue)").path

        do {
            try data.write(to: URL(fileURLWithPath: filePath))
            print("✅ \(format.rawValue.uppercased()) ファイルをエクスポート: \(filePath)")

            APIService.shared.uploadCADFile(data: data, format: format.rawValue) { success in
                if success {
                    print("☁️ クラウドにアップロード成功: \(format.rawValue.uppercased())")
                } else {
                    print("❌ クラウドアップロード失敗")
                }
                completion(success)
            }
        } catch {
            print("❌ エクスポート失敗: \(error.localizedDescription)")
            completion(false)
        }
    }
}
