import SwiftUI

struct ModeHighlight: ViewModifier {
    var isSelected: Bool
    
    func body(content: Content) -> some View {
        content
            .padding()
            .background(isSelected ? Color.blue.opacity(0.3) : Color.clear)
            .cornerRadius(10)
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(isSelected ? Color.blue : Color.gray, lineWidth: isSelected ? 2 : 1)
            )
    }
}
