//
//  UserProfileView.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import SwiftUI

struct UserProfileView: View {
    @EnvironmentObject var userAuth: UserAuth
    
    var body: some View {
        VStack {
            if let user = userAuth.currentUser {
                Text("こんにちは, \(user.displayName ?? "ユーザー")!")
                    .font(.title)
                    .padding()
                
                Button(action: {
                    userAuth.signOut()
                }) {
                    Text("ログアウト")
                        .foregroundColor(.white)
                        .padding()
                        .background(Color.red)
                        .cornerRadius(10)
                }
            } else {
                Text("ログインしてください")
                    .font(.headline)
                    .padding()
            }
        }
        .onAppear {
            userAuth.objectWillChange.send()
        }
    }
}
