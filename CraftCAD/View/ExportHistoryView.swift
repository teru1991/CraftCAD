import SwiftUI

struct ExportHistoryView: View {
    @State private var history = HistoryService.getExportHistory()

    var body: some View {
        VStack {
            Text("エクスポート履歴")
                .font(.headline)
                .padding()

            List {
                ForEach(history) { entry in
                    HStack {
                        VStack(alignment: .leading) {
                            Text(entry.fileName)
                                .font(.headline)
                            Text("形式: \(entry.format) ・ \(entry.date, formatter: DateFormatter.shortDate)")
                                .font(.subheadline)
                                .foregroundColor(.gray)
                        }
                        Spacer()

                        // ✅ 削除ボタン
                        Button(action: {
                            HistoryService.deleteExportHistory(entry: entry)
                            history = HistoryService.getExportHistory() // UI更新
                        }) {
                            Image(systemName: "trash")
                                .foregroundColor(.red)
                        }
                    }
                }
            }

            // ✅ 履歴全削除ボタン
            Button(action: {
                HistoryService.clearExportHistory()
                history = []
            }) {
                Text("履歴を全削除")
                    .foregroundColor(.white)
                    .padding()
                    .background(Color.red)
                    .cornerRadius(8)
            }
            .padding()
        }
    }
}

extension DateFormatter {
    static var shortDate: DateFormatter {
        let formatter = DateFormatter()
        formatter.dateStyle = .short
        return formatter
    }
}
