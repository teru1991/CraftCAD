import SwiftUI

struct UserInfoPanel: View {
    @EnvironmentObject var userAuth: UserAuth
    
    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            if let user = userAuth.currentUser {
                HStack {
                    Text("こんにちは, \(user.displayName ?? "ユーザー")!")
                        .font(.title2)
                        .bold()
                    Spacer()
                    Button(action: {
                        userAuth.signOut()
                    }) {
                        Text("ログアウト")
                            .foregroundColor(.white)
                            .padding()
                            .background(Color.red)
                            .cornerRadius(10)
                    }
                }
            } else {
                Text("ログインしてください")
                    .font(.headline)
                    .padding()
            }
        }
        .padding()
    }
}
