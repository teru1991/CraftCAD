use craftcad_commands::commands::create_shapes::*;
use craftcad_commands::{Command, CommandContext, History};
use craftcad_serialize::{Document, Layer, ProjectSettings};
use uuid::Uuid;

fn doc() -> (Document, Uuid) {
    let l = Uuid::new_v4();
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
            entities: vec![],
            parts: vec![],
            jobs: vec![],
            materials: vec![],
            settings: ProjectSettings::default(),
        },
        l,
    )
}

#[test]
fn rect_roundtrip() {
    let (mut d, l) = doc();
    let before = serde_json::to_value(&d).unwrap();
    let mut c = CreateRectCommand::new(l);
    c.begin(&CommandContext::default()).unwrap();
    c.update(CreateRectInput {
        params: RectParams::TwoPoint {
            p0: craftcad_serialize::Vec2 { x: 0.0, y: 0.0 },
            p1: craftcad_serialize::Vec2 { x: 10.0, y: 5.0 },
            corner: "Sharp".into(),
        },
    })
    .unwrap();
    let delta = c.commit().unwrap();
    delta.apply(&mut d).unwrap();
    let mut h = History::new();
    h.push(delta);
    h.undo(&mut d).unwrap();
    assert_eq!(serde_json::to_value(&d).unwrap(), before);
    h.redo(&mut d).unwrap();
    assert_eq!(d.entities.len(), 1);
}

#[test]
fn circle_arc_polyline_roundtrip() {
    let (mut d, l) = doc();
    let before = serde_json::to_value(&d).unwrap();
    let mut h = History::new();

    let mut c1 = CreateCircleCommand::new(l);
    c1.begin(&CommandContext::default()).unwrap();
    c1.update(CreateCircleInput {
        params: CircleParams::CenterRadius {
            c: craftcad_serialize::Vec2 { x: 0.0, y: 0.0 },
            r: 2.0,
        },
    })
    .unwrap();
    let d1 = c1.commit().unwrap();
    d1.apply(&mut d).unwrap();
    h.push(d1);

    let mut c2 = CreateArcCommand::new(l);
    c2.begin(&CommandContext::default()).unwrap();
    c2.update(CreateArcInput {
        params: ArcParams::Center {
            c: craftcad_serialize::Vec2 { x: 1.0, y: 1.0 },
            r: 1.0,
            start_angle: 0.0,
            end_angle: 1.0,
            ccw: true,
        },
    })
    .unwrap();
    let d2 = c2.commit().unwrap();
    d2.apply(&mut d).unwrap();
    h.push(d2);

    let mut c3 = CreatePolylineCommand::new(l);
    c3.begin(&CommandContext::default()).unwrap();
    c3.update(CreatePolylineInput {
        params: PolylineParams {
            pts: vec![
                craftcad_serialize::Vec2 { x: 0.0, y: 0.0 },
                craftcad_serialize::Vec2 { x: 1.0, y: 0.0 },
                craftcad_serialize::Vec2 { x: 1.0, y: 1.0 },
            ],
            closed: false,
        },
    })
    .unwrap();
    let d3 = c3.commit().unwrap();
    d3.apply(&mut d).unwrap();
    h.push(d3);

    h.undo(&mut d).unwrap();
    h.undo(&mut d).unwrap();
    h.undo(&mut d).unwrap();
    assert_eq!(serde_json::to_value(&d).unwrap(), before);
}
