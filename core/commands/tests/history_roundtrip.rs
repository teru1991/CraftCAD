use craftcad_commands::commands::create_line::{CreateLineCommand, CreateLineInput};
use craftcad_commands::{Command, CommandContext, History};
use craftcad_serialize::{Document, Layer, Vec2};
use uuid::Uuid;

fn sample_doc() -> Document {
    let layer_id = Uuid::new_v4();
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
        entities: vec![],
        parts: vec![],
        jobs: vec![],
    }
}

fn doc_json(doc: &Document) -> serde_json::Value {
    serde_json::to_value(doc).expect("serialize")
}

#[test]
fn creates_an_entity_on_commit_apply() {
    let mut doc = sample_doc();
    let layer_id = doc.layers[0].id;
    let mut cmd = CreateLineCommand::new(layer_id);

    cmd.begin(&CommandContext::default()).expect("begin");
    cmd.update(CreateLineInput {
        a: Vec2 { x: 0.0, y: 0.0 },
        b: Vec2 { x: 10.0, y: 10.0 },
    })
    .expect("update");

    let delta = cmd.commit().expect("commit");
    delta.apply(&mut doc).expect("apply");

    assert_eq!(doc.entities.len(), 1);
}

#[test]
fn undo_redo_roundtrip_returns_exact_json_state() {
    let mut doc = sample_doc();
    let layer_id = doc.layers[0].id;
    let before = doc_json(&doc);
    let mut cmd = CreateLineCommand::new(layer_id);
    let mut history = History::new();

    cmd.begin(&CommandContext::default()).expect("begin");
    cmd.update(CreateLineInput {
        a: Vec2 { x: 1.0, y: 2.0 },
        b: Vec2 { x: 3.0, y: 4.0 },
    })
    .expect("update");

    let delta = cmd.commit().expect("commit");
    delta.apply(&mut doc).expect("apply");
    history.push(delta);

    history.undo(&mut doc).expect("undo");
    assert_eq!(doc_json(&doc), before);

    history.redo(&mut doc).expect("redo");
    assert_eq!(doc.entities.len(), 1);
}

#[test]
fn grouped_changes_are_undone_in_one_step() {
    let mut doc = sample_doc();
    let layer_id = doc.layers[0].id;
    let before = doc_json(&doc);
    let mut history = History::new();

    history.begin_group("drag");
    for idx in 0..2 {
        let mut cmd = CreateLineCommand::new(layer_id);
        cmd.begin(&CommandContext::default()).expect("begin");
        cmd.update(CreateLineInput {
            a: Vec2 {
                x: idx as f64,
                y: 0.0,
            },
            b: Vec2 { x: 10.0, y: 5.0 },
        })
        .expect("update");

        let delta = cmd.commit().expect("commit");
        delta.apply(&mut doc).expect("apply");
        history.push(delta);
    }
    history.end_group();

    assert_eq!(doc.entities.len(), 2);
    history.undo(&mut doc).expect("undo");
    assert_eq!(doc_json(&doc), before);
}
