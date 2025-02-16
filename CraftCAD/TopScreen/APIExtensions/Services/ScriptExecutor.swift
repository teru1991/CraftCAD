import Foundation

class ScriptExecutor {
    enum ScriptType {
        case python
        case javascript
    }

    func execute(script: String, type: ScriptType) -> String {
        #if os(macOS)
        let process = Process()
        let outputPipe = Pipe()
        let errorPipe = Pipe()

        switch type {
        case .python:
            process.executableURL = URL(fileURLWithPath: "/usr/bin/python3")
        case .javascript:
            process.executableURL = URL(fileURLWithPath: "/usr/bin/node")
        }

        process.arguments = ["-c", script]
        process.standardOutput = outputPipe
        process.standardError = errorPipe

        do {
            try process.run()
            process.waitUntilExit()

            let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let errorData = errorPipe.fileHandleForReading.readDataToEndOfFile()

            let outputString = String(data: outputData, encoding: .utf8) ?? ""
            let errorString = String(data: errorData, encoding: .utf8) ?? ""

            if process.terminationStatus == 0 {
                return outputString.isEmpty ? "スクリプトの出力なし" : outputString
            } else {
                return "エラー: \(errorString)"
            }
        } catch {
            return "エラー: スクリプトの実行に失敗しました (\(error.localizedDescription))"
        }
        #else
        return "エラー: iOS ではスクリプト実行はサポートされていません"
        #endif
    }
}
