import SwiftUI

struct ScriptExecutorView: View {
    @State private var scriptType: ScriptExecutor.ScriptType = .python
    @State private var scriptCode: String = ""
    @State private var output: String = ""

    var body: some View {
        VStack {
            Text("スクリプト実行")
                .font(.title)
                .padding()

            Picker("スクリプト言語", selection: $scriptType) {
                Text("Python").tag(ScriptExecutor.ScriptType.python)
                Text("JavaScript").tag(ScriptExecutor.ScriptType.javascript)
            }
            .pickerStyle(SegmentedPickerStyle())
            .padding()

            TextEditor(text: $scriptCode)
                .border(Color.gray, width: 1)
                .frame(height: 200)
                .padding()

            Button("実行") {
                // `if let` を削除し、直接代入する
                let result = ScriptExecutor().execute(script: scriptCode, type: scriptType)
                output = result.isEmpty ? "エラー: スクリプト実行失敗" : result
            }
            .padding()

            Text("出力:")
            Text(output)
                .border(Color.gray, width: 1)
                .frame(height: 100)
                .padding()
        }
    }
}
