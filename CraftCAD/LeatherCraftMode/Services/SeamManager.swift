import Foundation

class SeamManager {
    var showSeams: Bool = true

    func detectSeams(for part: LeatherPart) -> [(Int, Int)] {
        return part.edges.filter { _ in Bool.random() }
    }

    func toggleSeamVisibility() {
        showSeams.toggle()
    }
}
