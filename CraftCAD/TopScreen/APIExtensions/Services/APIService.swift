import Foundation

class APIService {
    static let shared = APIService()
    
    private init() {}

    func uploadCADFile(data: Data, format: String, completion: @escaping (Bool) -> Void) {
        let serverURL = URL(string: "https://example.com/upload")!
        var request = URLRequest(url: serverURL)
        request.httpMethod = "POST"
        request.setValue("application/\(format)", forHTTPHeaderField: "Content-Type")
        request.httpBody = data

        let task = URLSession.shared.dataTask(with: request) { _, response, error in
            if let error = error {
                ErrorHandler.handleError(error, context: "クラウドアップロード")
                completion(false)
            } else if let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 {
                print("✅ \(format.uppercased()) ファイルがクラウドにアップロードされました")
                completion(true)
            } else {
                completion(false)
            }
        }
        task.resume()
    }

    func downloadCADFile(from url: String, completion: @escaping (Data?) -> Void) {
        guard let fileURL = URL(string: url) else {
            print("❌ 無効なURL")
            completion(nil)
            return
        }

        let task = URLSession.shared.dataTask(with: fileURL) { data, _, error in
            if let error = error {
                ErrorHandler.handleError(error, context: "クラウドダウンロード")
                completion(nil)
            } else {
                completion(data)
            }
        }
        task.resume()
    }
}
