use craftcad_commands::commands::create_part::{CreatePartCommand, CreatePartInput};
use craftcad_commands::{Command, CommandContext, History};
use craftcad_serialize::{Document, Layer, Part, Polygon2D};
use uuid::Uuid;

fn sample_doc() -> Document {
    let layer_id = Uuid::new_v4();
    Document {
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
        entities: vec![],
        parts: vec![],
        jobs: vec![],
    }
}

#[test]
fn create_part_undo_redo_json_roundtrip() {
    let mut doc = sample_doc();
    let before = serde_json::to_value(&doc).unwrap();
    let part = Part {
        id: Uuid::new_v4(),
        name: "P1".into(),
        outline: Polygon2D {
            outer: vec![
                craftcad_serialize::Vec2 { x: 0.0, y: 0.0 },
                craftcad_serialize::Vec2 { x: 10.0, y: 0.0 },
                craftcad_serialize::Vec2 { x: 10.0, y: 10.0 },
                craftcad_serialize::Vec2 { x: 0.0, y: 10.0 },
            ],
            holes: vec![],
        },
        thickness: 1.0,
        quantity: 1,
        material_id: Uuid::new_v4(),
        grain_dir: None,
        allow_rotate: true,
        margin: 0.0,
        kerf: 0.0,
    };

    let mut cmd = CreatePartCommand::new();
    let mut history = History::new();
    cmd.begin(&CommandContext::default()).unwrap();
    cmd.update(CreatePartInput { part }).unwrap();
    let delta = cmd.commit().unwrap();
    delta.apply(&mut doc).unwrap();
    history.push(delta);
    history.undo(&mut doc).unwrap();
    assert_eq!(serde_json::to_value(&doc).unwrap(), before);
    history.redo(&mut doc).unwrap();
    assert_eq!(doc.parts.len(), 1);
}
