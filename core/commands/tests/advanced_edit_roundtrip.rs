use craftcad_commands::commands::advanced_edit::*;
use craftcad_commands::{Command, CommandContext, History};
use craftcad_serialize::{Document, Entity, Geom2D, Layer, ProjectSettings, Vec2};
use std::collections::BTreeMap;
use uuid::Uuid;

fn doc() -> (Document, Uuid, Uuid, Uuid) {
    let l = Uuid::new_v4();
    let e1 = Uuid::new_v4();
    let e2 = Uuid::new_v4();
    (
        Document {
            schema_version: 1,
            id: Uuid::new_v4(),
            units: "mm".into(),
            layers: vec![Layer {
                id: l,
                name: "L".into(),
                visible: true,
                locked: false,
                editable: true,
            }],
            entities: vec![
                Entity {
                    id: e1,
                    layer_id: l,
                    geom: Geom2D::Line {
                        a: Vec2 { x: 0.0, y: 0.0 },
                        b: Vec2 { x: 10.0, y: 0.0 },
                    },
                    style: serde_json::json!({}),
                    tags: vec![],
                    meta: BTreeMap::new(),
                },
                Entity {
                    id: e2,
                    layer_id: l,
                    geom: Geom2D::Line {
                        a: Vec2 { x: 0.0, y: 0.0 },
                        b: Vec2 { x: 0.0, y: 10.0 },
                    },
                    style: serde_json::json!({}),
                    tags: vec![],
                    meta: BTreeMap::new(),
                },
            ],
            parts: vec![],
            jobs: vec![],
            materials: vec![],
            settings: ProjectSettings::default(),
        },
        l,
        e1,
        e2,
    )
}

#[test]
fn fillet_chamfer_mirror_pattern_roundtrip() {
    let (mut d, _l, e1, e2) = doc();
    let before = serde_json::to_value(&d).unwrap();
    let mut h = History::new();

    let mut fc = FilletCommand::new();
    fc.begin(&CommandContext::default()).unwrap();
    fc.update(FilletInput {
        e1,
        e2,
        radius: 1.0,
    })
    .unwrap();
    let df = fc.commit().unwrap();
    df.apply(&mut d).unwrap();
    h.push(df);
    h.undo(&mut d).unwrap();
    assert_eq!(serde_json::to_value(&d).unwrap(), before);
    h.redo(&mut d).unwrap();

    let mut mc = MirrorCommand::new();
    mc.begin(&CommandContext::default()).unwrap();
    mc.update(MirrorInput {
        selection_ids: vec![e1],
        axis_a: Vec2 { x: 0.0, y: 0.0 },
        axis_b: Vec2 { x: 0.0, y: 1.0 },
    })
    .unwrap();
    let dm = mc.commit().unwrap();
    dm.apply(&mut d).unwrap();
    h.push(dm);

    let mut pc = PatternCommand::new();
    pc.begin(&CommandContext::default()).unwrap();
    pc.update(PatternInput {
        selection_ids: vec![e1],
        params: PatternParams::Linear {
            dx: 1.0,
            dy: 0.0,
            count: 3,
        },
    })
    .unwrap();
    let dp = pc.commit().unwrap();
    dp.apply(&mut d).unwrap();
    h.push(dp);
}
