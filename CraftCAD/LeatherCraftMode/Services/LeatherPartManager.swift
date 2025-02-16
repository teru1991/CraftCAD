import Foundation

class LeatherPartManager {
    func splitPart(_ part: LeatherPart, at index: Int) -> (LeatherPart, LeatherPart)? {
        return part.split(at: index)
    }

    func mergeParts(_ part1: inout LeatherPart, _ part2: LeatherPart) {
        part1.merge(with: part2)
    }
}

