import SwiftUI

struct ProjectList: View {
    @ObservedObject var projectViewModel = ProjectViewModel()
    
    var body: some View {
        VStack(alignment: .leading) {
            Text("最近のプロジェクト")
                .font(.headline)
                .padding(.leading)
            
            List(projectViewModel.projects) { project in
                HStack {
                    Text(project.name)
                    Spacer()
                    Text(project.lastModified, style: .date)
                        .foregroundColor(.gray)
                }
                .onTapGesture {
                    projectViewModel.openProject(project)
                }
            }
            .onAppear {
                projectViewModel.fetchProjects()
            }
        }
    }
}
