import SwiftUI

struct ProjectListView: View {
    @ObservedObject var projectManager = ProjectManager.shared
    
    var body: some View {
        NavigationView {
            VStack {
                Text("プロジェクト一覧")
                    .font(.title)
                    .padding()
                
                List {
                    ForEach(projectManager.projects) { project in
                        HStack {
                            Text(project.name)
                            Spacer()
                            Text(project.lastModified, format: .dateTime)
                                .foregroundColor(.gray)
                        }
                        .onTapGesture {
                            print("Opening project: \(project.name)")
                        }
                    }
                    .onDelete { indexSet in
                        indexSet.forEach { index in
                            let projectID = projectManager.projects[index].id
                            projectManager.deleteProject(id: projectID)
                        }
                    }
                }
                
                Button(action: {
                    projectManager.createProject(name: "新しいプロジェクト")
                }) {
                    Text("新しいプロジェクトを作成")
                        .padding()
                        .background(Color.blue)
                        .foregroundColor(.white)
                        .cornerRadius(8)
                }
                .padding()
            }
            .navigationTitle("プロジェクト管理")
        }
    }
}
