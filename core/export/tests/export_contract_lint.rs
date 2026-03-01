use craftcad_export::{compute_tiled_layout, TiledPdfOptions};
use craftcad_serialize::{Document, ProjectSettings};
use uuid::Uuid;

#[test]
fn export_options_schema_exists_and_has_id() {
    let raw = std::fs::read_to_string("schemas/export_options.schema.json")
        .or_else(|_| std::fs::read_to_string("../export/schemas/export_options.schema.json"))
        .expect("schema file");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("json");
    assert!(v.get("$id").and_then(|x| x.as_str()).is_some());
}

#[test]
fn tiled_defaults_invariants() {
    let doc = Document {
        schema_version: 1,
        id: Uuid::new_v4(),
        units: "mm".to_string(),
        layers: vec![],
        entities: vec![],
        parts: vec![],
        jobs: vec![],
        materials: vec![],
        settings: ProjectSettings {
            bom_delimiter: None,
        },
    };
    let layout = compute_tiled_layout(&doc, &TiledPdfOptions::default()).expect("layout");
    assert!(layout.tiles_x >= 1 && layout.tiles_y >= 1);
}
