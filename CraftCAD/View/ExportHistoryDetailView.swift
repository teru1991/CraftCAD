import SwiftUI

struct ExportHistoryDetailView: View {
    let entry: ExportHistoryEntry

    // モーダルを閉じるための環境変数
    @Environment(\.dismiss) var dismiss

    var body: some View {
        NavigationView {
            VStack(alignment: .leading, spacing: 16) {
                Text("ファイル名: \(entry.fileName)")
                    .font(.title2)
                    .fontWeight(.bold)
                Text("形式: \(entry.format)")
                    .font(.headline)
                Text("エクスポート日時: \(entry.date, formatter: DateFormatter.shortDate)")
                    .font(.subheadline)
                    .foregroundColor(.gray)
                
                // 他に表示したい詳細情報があればここに追加

                Spacer()
            }
            .padding()
            .navigationTitle("エクスポート詳細")
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("閉じる") {
                        dismiss()
                    }
                }
            }
        }
    }
}

struct ExportHistoryDetailView_Previews: PreviewProvider {
    static var previews: some View {
        // プレビュー用のダミーデータ
        let sampleEntry = ExportHistoryEntry(
            fileName: "sample.pdf",
            format: "PDF",
            date: Date(),
            fileURL: URL(string: "https://example.com")!
        )
        ExportHistoryDetailView(entry: sampleEntry)
    }
}
