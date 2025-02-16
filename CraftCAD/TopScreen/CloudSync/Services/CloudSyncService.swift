import Foundation

/// クラウドとの通信を管理するサービス
class CloudSyncService: ObservableObject {
    static let shared = CloudSyncService()
    
    private let adapter = CloudStorageAdapter()
    
    @Published var isSyncing: Bool = false // UI更新用の状態管理

    /// ファイルをアップロード
    func uploadFile(localPath: String, remotePath: String, completion: @escaping (Bool) -> Void) {
        adapter.uploadFile(localPath: localPath, remotePath: remotePath) { success in
            DispatchQueue.main.async {
                self.objectWillChange.send() // SwiftUIに更新を通知
                completion(success)
            }
        }
    }
    
    /// ファイルをダウンロード
    func downloadFile(remotePath: String, localPath: String, completion: @escaping (Bool) -> Void) {
        adapter.downloadFile(remotePath: remotePath, localPath: localPath) { success in
            DispatchQueue.main.async {
                self.objectWillChange.send() // SwiftUIに更新を通知
                completion(success)
            }
        }
    }
    
    /// クラウドとの同期処理を実行
    func syncData(completion: @escaping (Bool) -> Void) {
        DispatchQueue.main.async {
            self.isSyncing = true
        }
        
        print("クラウドとの同期を開始...")
        adapter.syncAll { success in
            DispatchQueue.main.async {
                self.isSyncing = false
                self.objectWillChange.send() // SwiftUIに更新を通知
                completion(success)
            }
        }
    }
}
