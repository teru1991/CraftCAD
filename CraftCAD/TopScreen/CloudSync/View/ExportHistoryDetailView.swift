import SwiftUI
import Foundation

struct ExportHistoryDetailView: View {
    let entry: ExportHistoryEntry

    @Environment(\.dismiss) var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("ファイル名: \(entry.fileName)")
                .font(.title2)
                .fontWeight(.bold)
            Text("形式: \(entry.format)")
                .font(.headline)
            Text("エクスポート日時: \(entry.date.formatted(date: .abbreviated, time: .shortened))")
                .font(.subheadline)
                .foregroundColor(.gray)

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

// プレビュー
struct ExportHistoryDetailView_Previews: PreviewProvider {
    static var previews: some View {
        let sampleEntry = ExportHistoryEntry(
            fileName: "sample.pdf",
            format: "PDF",
            date: Date(),
            fileURL: URL(string: "https://example.com")!
        )
        ExportHistoryDetailView(entry: sampleEntry)
    }
}
