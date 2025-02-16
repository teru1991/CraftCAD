//
//  HistoryService.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/16.
//


import Foundation

/// エクスポート履歴の管理を行うサービス
class HistoryService {
    /// サンプルのエクスポート履歴を返す（仮実装）
    static func getExportHistory() -> [ExportHistoryEntry] {
        return [
            ExportHistoryEntry(
                fileName: "design1.pdf",
                format: "PDF",
                date: Date(),
                fileURL: URL(string: "https://example.com/design1.pdf")!
            ),
            ExportHistoryEntry(
                fileName: "design2.dxf",
                format: "DXF",
                date: Date().addingTimeInterval(-86400), // 1日前
                fileURL: URL(string: "https://example.com/design2.dxf")!
            )
        ]
    }
}
