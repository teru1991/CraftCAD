//
//  SyncManager.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import Foundation

/// クラウド同期のスケジュールを管理するクラス
class SyncManager {
    static let shared = SyncManager()
    
    private var timer: Timer?
    
    /// 一定時間ごとに同期を実行
    func startAutoSync(interval: TimeInterval = 3600) {
        stopAutoSync()
        timer = Timer.scheduledTimer(withTimeInterval: interval, repeats: true) { _ in
            CloudSyncService.shared.syncData { success in
                print("自動同期完了: \(success)")
            }
        }
    }
    
    /// 自動同期を停止
    func stopAutoSync() {
        timer?.invalidate()
        timer = nil
    }
    
    /// 手動で同期を実行
    func manualSync() {
        CloudSyncService.shared.syncData { success in
            print("手動同期完了: \(success)")
        }
    }
}
