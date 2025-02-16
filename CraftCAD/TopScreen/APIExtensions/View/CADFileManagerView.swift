import SwiftUI

typealias CADFormat = CADIntegration.CADFormat

struct CADFileManagerView: View {
    @State private var selectedFormat: CADFormat = .dxf
    @State private var filePath: String = ""
    @State private var cloudFileURL: String = ""
    @State private var statusMessage: String = ""

    var body: some View {
        VStack {
            Text("CAD ファイル管理")
                .font(.title)
                .padding()

            Picker("ファイル形式", selection: $selectedFormat) {
                ForEach(CADIntegration.CADFormat.allCases, id: \.self) { format in
                    Text(format.rawValue.uppercased()).tag(format)
                }
            }
            .pickerStyle(MenuPickerStyle())
            .padding()

            TextField("ローカルファイルパス", text: $filePath)
                .textFieldStyle(RoundedBorderTextFieldStyle())
                .padding()

            HStack {
                Button("インポート") {
                    if let data = CADIntegration.importData(from: filePath) {
                        statusMessage = "✅ インポート成功: \(filePath)"
                    } else {
                        statusMessage = "❌ インポート失敗"
                    }
                }

                Button("エクスポート") {
                    let sampleData = "Sample CAD Data".data(using: .utf8)!
                    APIService.shared.uploadCADFile(data: sampleData, format: selectedFormat.rawValue) { success in
                        statusMessage = success ? "☁️ クラウドにアップロード成功" : "❌ クラウドアップロード失敗"
                    }
                }
            }
            .padding()

            TextField("クラウドから取得するURL", text: $cloudFileURL)
                .textFieldStyle(RoundedBorderTextFieldStyle())
                .padding()

            Button("クラウドからダウンロード") {
                APIService.shared.downloadCADFile(from: cloudFileURL) { data in
                    if let data = data {
                        statusMessage = "✅ クラウドデータ取得成功"
                    } else {
                        statusMessage = "❌ クラウドデータ取得失敗"
                    }
                }
            }

            Text(statusMessage)
                .padding()
        }
    }
}
