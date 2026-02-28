use craftcad_commands::commands::offset_entity::{OffsetEntityCommand, OffsetEntityInput};
use craftcad_commands::commands::trim_entity::{TrimEntityCommand, TrimEntityInput};
use craftcad_commands::{Command, CommandContext, History};
use craftcad_serialize::{Document, Entity, Geom2D, Layer, Vec2};
use diycad_geom::EpsilonPolicy;
use std::collections::BTreeMap;
use uuid::Uuid;

fn base_doc() -> (Document, Uuid, Uuid) {
    let layer_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    let cutter_id = Uuid::new_v4();
    (
        Document {
            schema_version: 1,
            id: Uuid::new_v4(),
            units: "mm".to_string(),
            layers: vec![Layer {
                id: layer_id,
                name: "Default".into(),
                visible: true,
                locked: false,
                editable: true,
            }],
            entities: vec![
                Entity {
                    id: target_id,
                    layer_id,
                    geom: Geom2D::Line {
                        a: Vec2 { x: 0.0, y: 0.0 },
                        b: Vec2 { x: 10.0, y: 0.0 },
                    },
                    style: serde_json::json!({}),
                    tags: vec![],
                    meta: BTreeMap::new(),
                },
                Entity {
                    id: cutter_id,
                    layer_id,
                    geom: Geom2D::Line {
                        a: Vec2 { x: 5.0, y: -10.0 },
                        b: Vec2 { x: 5.0, y: 10.0 },
                    },
                    style: serde_json::json!({}),
                    tags: vec![],
                    meta: BTreeMap::new(),
                },
            ],
            parts: vec![],
            jobs: vec![],
        },
        target_id,
        cutter_id,
    )
}

fn doc_json(doc: &Document) -> serde_json::Value {
    serde_json::to_value(doc).expect("serialize")
}

#[test]
fn offset_line_and_history_roundtrip() {
    let (mut doc, target_id, _) = base_doc();
    let before = doc_json(&doc);
    let mut cmd = OffsetEntityCommand::new();
    let mut history = History::new();

    cmd.begin(&CommandContext::default()).unwrap();
    cmd.update(OffsetEntityInput {
        entity_id: target_id,
        dist: 2.0,
        eps: EpsilonPolicy::default(),
    })
    .unwrap();
    let delta = cmd.commit().unwrap();
    delta.apply(&mut doc).unwrap();
    history.push(delta);

    assert_eq!(doc.entities.len(), 3);
    history.undo(&mut doc).unwrap();
    assert_eq!(doc_json(&doc), before);
    history.redo(&mut doc).unwrap();
    assert_eq!(doc.entities.len(), 3);
}

#[test]
fn trim_line_and_history_roundtrip() {
    let (mut doc, target_id, cutter_id) = base_doc();
    let before = doc_json(&doc);
    let mut cmd = TrimEntityCommand::new();
    let mut history = History::new();

    cmd.begin(&CommandContext::default()).unwrap();
    cmd.update(TrimEntityInput {
        entity_id: target_id,
        cutter_id,
        pick_point: Vec2 { x: 9.0, y: 0.0 },
        eps: EpsilonPolicy::default(),
        candidate_index: None,
    })
    .unwrap();

    let delta = cmd.commit().unwrap();
    delta.apply(&mut doc).unwrap();
    history.push(delta);

    history.undo(&mut doc).unwrap();
    assert_eq!(doc_json(&doc), before);
}

#[test]
fn trim_ambiguous_returns_candidates() {
    let (mut doc, target_id, _) = base_doc();
    let layer_id = doc.layers[0].id;
    // cutter polyline intersects target at x=3 and x=7, pick at x=5 makes ambiguity
    doc.entities.push(Entity {
        id: Uuid::new_v4(),
        layer_id,
        geom: Geom2D::Polyline {
            pts: vec![
                Vec2 { x: 3.0, y: -5.0 },
                Vec2 { x: 3.0, y: 5.0 },
                Vec2 { x: 7.0, y: -5.0 },
                Vec2 { x: 7.0, y: 5.0 },
            ],
            closed: false,
        },
        style: serde_json::json!({}),
        tags: vec![],
        meta: BTreeMap::new(),
    });
    let cutter_id = doc.entities.last().unwrap().id;

    let mut cmd = TrimEntityCommand::new();
    cmd.begin(&CommandContext::default()).unwrap();
    cmd.update(TrimEntityInput {
        entity_id: target_id,
        cutter_id,
        pick_point: Vec2 { x: 5.0, y: 0.0 },
        eps: EpsilonPolicy::default(),
        candidate_index: Some(9),
    })
    .unwrap();
    let delta = cmd.commit().unwrap();
    let err = delta.apply(&mut doc).expect_err("expected ambiguity");
    assert_eq!(err.code, "EDIT_TRIM_AMBIGUOUS_CANDIDATE");
    assert!(err.debug.get("candidates").is_some());
}
