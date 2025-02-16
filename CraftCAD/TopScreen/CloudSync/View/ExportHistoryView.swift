import SwiftUI

struct ExportHistoryView: View {
    /// エクスポート履歴の取得
    @State private var history: [ExportHistoryEntry] = HistoryService.getExportHistory()

    var body: some View {
        NavigationView {
            List(history) { entry in
                NavigationLink(destination: ExportHistoryDetailView(entry: entry)) {
                    HStack {
                        VStack(alignment: .leading) {
                            Text(entry.fileName)
                                .font(.headline)
                            Text("形式: \(entry.format) ・ \(entry.date.formatted(date: .abbreviated, time: .shortened))")
                                .font(.subheadline)
                                .foregroundColor(.gray)
                        }
                        Spacer()
                    }
                    .padding(.vertical, 4)
                }
            }
            .navigationTitle("エクスポート履歴")
        }
    }
}

// プレビュー
struct ExportHistoryView_Previews: PreviewProvider {
    static var previews: some View {
        ExportHistoryView()
    }
}

// `ExportHistoryView` の初期化を外部から行う場合
extension ExportHistoryView {
    init(history: [ExportHistoryEntry]) {
        self._history = State(initialValue: history)
    }
}
