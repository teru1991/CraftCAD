import SwiftUI

struct StorageStatus: View {
    @ObservedObject var storageViewModel = StorageViewModel()
    
    var body: some View {
        VStack(alignment: .leading) {
            Text("ストレージ使用状況")
                .font(.headline)
                .padding(.leading)
            
            HStack {
                ProgressView(value: storageViewModel.usage, total: storageViewModel.total)
                    .progressViewStyle(LinearProgressViewStyle())
                    .frame(maxWidth: .infinity)
                
                Text("\(Int(storageViewModel.usage)) / \(Int(storageViewModel.total)) GB")
                    .foregroundColor(.gray)
            }
            .padding()
            
            Button("手動同期") {
                storageViewModel.syncStorage()
            }
            .buttonStyle(.bordered)
            .padding()
        }
        .onAppear {
            storageViewModel.fetchStorageStatus()
        }
    }
}
