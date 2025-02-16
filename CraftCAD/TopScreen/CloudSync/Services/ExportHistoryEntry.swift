

import Foundation

struct ExportHistoryEntry: Identifiable {
    let id = UUID()
    let fileName: String
    let format: String
    let date: Date
    let fileURL: URL
}

