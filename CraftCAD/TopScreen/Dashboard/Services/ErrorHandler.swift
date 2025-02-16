
import Foundation

class ErrorHandler {
    static func handleError(_ error: Error, context: String) {
        let errorMessage = "❌ [\(context)] \(error.localizedDescription)"
        LogManager.shared.addLog(errorMessage)
        print(errorMessage)
    }
}
