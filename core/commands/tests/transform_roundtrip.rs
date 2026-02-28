use craftcad_commands::commands::transform_selection::{
    Transform, TransformSelectionCommand, TransformSelectionInput,
};
use craftcad_commands::{Command, CommandContext, History};
use craftcad_serialize::{Document, Entity, Geom2D, Layer, Vec2};
use std::collections::BTreeMap;
use uuid::Uuid;

fn sample_doc() -> (Document, Uuid) {
    let layer_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    (
        Document {
            schema_version: 1,
            id: Uuid::new_v4(),
            units: "mm".to_string(),
            layers: vec![Layer {
                id: layer_id,
                name: "Default".to_string(),
                visible: true,
                locked: false,
                editable: true,
            }],
            entities: vec![Entity {
                id: entity_id,
                layer_id,
                geom: Geom2D::Line {
                    a: Vec2 { x: 0.0, y: 0.0 },
                    b: Vec2 { x: 10.0, y: 0.0 },
                },
                style: serde_json::json!({}),
                tags: vec![],
                meta: BTreeMap::new(),
            }],
            parts: vec![],
            jobs: vec![],
        },
        entity_id,
    )
}

fn doc_json(doc: &Document) -> serde_json::Value {
    serde_json::to_value(doc).expect("serialize")
}

#[test]
fn transform_roundtrip_undo_redo_json_equality() {
    let (mut doc, entity_id) = sample_doc();
    let before = doc_json(&doc);
    let mut cmd = TransformSelectionCommand::new();
    let mut history = History::new();

    cmd.begin(&CommandContext::default()).expect("begin");
    cmd.update(TransformSelectionInput {
        selection_ids: vec![entity_id],
        transform: Transform::Rotate {
            cx: 0.0,
            cy: 0.0,
            angle_rad: std::f64::consts::FRAC_PI_2,
        },
    })
    .expect("update");

    let delta = cmd.commit().expect("commit");
    delta.apply(&mut doc).expect("apply");
    history.push(delta);

    history.undo(&mut doc).expect("undo");
    assert_eq!(doc_json(&doc), before);

    history.redo(&mut doc).expect("redo");
    history.undo(&mut doc).expect("undo 2");
    assert_eq!(doc_json(&doc), before);
}
