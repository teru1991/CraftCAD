import SwiftUI

struct MaterialPicker: View {
    var applyMaterial: (UIColor?, UIImage?) -> Void

    let colors: [UIColor] = [.red, .blue, .green, .yellow, .purple, .brown]
    
    let textures: [UIImage?] = [
        UIImage(named: "leather_texture"),
        UIImage(named: "wood_texture"),
        UIImage(named: "metal_texture")
    ].compactMap { $0 } // nil を除外
    
    var body: some View {
        VStack {
            Text("色を選択")
            HStack {
                ForEach(colors, id: \.self) { color in
                    Button(action: {
                        applyMaterial(color, nil)
                    }) {
                        Circle()
                            .fill(Color(color))
                            .frame(width: 40, height: 40)
                    }
                }
            }
            
            Text("テクスチャを選択")
            HStack {
                ForEach(textures, id: \.self) { texture in
                    Button(action: {
                        applyMaterial(nil, texture)
                    }) {
                        Image(uiImage: texture)
                            .resizable()
                            .frame(width: 40, height: 40)
                            .clipShape(Circle())
                    }
                }
            }
        }
        .padding()
    }
}
