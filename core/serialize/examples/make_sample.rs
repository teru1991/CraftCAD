use craftcad_serialize::{create_manifest, save_diycad, Document, Entity, Geom2D, Layer, Vec2};
use std::collections::BTreeMap;
use std::path::Path;
use uuid::Uuid;

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sample.diycad".into());
    let layer_id = Uuid::new_v4();
    let doc = Document {
        schema_version: 1,
        id: Uuid::new_v4(),
        units: "mm".into(),
        layers: vec![Layer {
            id: layer_id,
            name: "Default".into(),
            visible: true,
            locked: false,
            editable: true,
        }],
        entities: vec![Entity {
            id: Uuid::new_v4(),
            layer_id,
            geom: Geom2D::Line {
                a: Vec2 { x: 0.0, y: 0.0 },
                b: Vec2 { x: 100.0, y: 20.0 },
            },
            style: serde_json::json!({}),
            tags: vec![],
            meta: BTreeMap::new(),
        }],
        parts: vec![],
        jobs: vec![],
        materials: vec![],
        settings: craftcad_serialize::ProjectSettings::default(),
    };
    let manifest = create_manifest("CraftCAD", "0.1.0");
    save_diycad(Path::new(&out), &manifest, &doc).expect("save sample");
    println!("wrote {}", out);
}
