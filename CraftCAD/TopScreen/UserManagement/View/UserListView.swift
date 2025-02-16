//
//  UserListView.swift
//  CraftCAD
//
//  Created by 板橋慶治 on 2025/02/05.
//


import SwiftUI

struct UserListView: View {
    @ObservedObject var userViewModel = UserViewModel()
    
    var body: some View {
        List(userViewModel.users) { user in
            HStack {
                Text(user.name)
                Spacer()
                Text(user.role.rawValue)
                    .foregroundColor(user.role == .admin ? .red : .gray)
            }
        }
        .navigationTitle("ユーザー管理")
        .onAppear {
            userViewModel.fetchUsers()
        }
    }
}
