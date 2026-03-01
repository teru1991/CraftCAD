use craftcad_commands::commands::nesting::{RunNestingCommand, RunNestingInput};
use craftcad_commands::{Command, CommandContext, History};
use craftcad_serialize::*;
use diycad_geom::EpsilonPolicy;
use diycad_nesting::RunLimits;
use uuid::Uuid;

fn rect(x: f64, y: f64, w: f64, h: f64) -> Polygon2D {
    Polygon2D {
        outer: vec![
            Vec2 { x, y },
            Vec2 { x: x + w, y },
            Vec2 { x: x + w, y: y + h },
            Vec2 { x, y: y + h },
        ],
        holes: vec![],
    }
}

fn mk_doc() -> (Document, Uuid) {
    let material_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let job_id = Uuid::new_v4();
    let doc = Document {
        schema_version: 1,
        id: Uuid::new_v4(),
        units: "mm".into(),
        layers: vec![],
        entities: vec![],
        parts: vec![Part {
            id: part_id,
            name: "P".into(),
            outline: rect(0.0, 0.0, 10.0, 10.0),
            thickness: 3.0,
            quantity: 1,
            material_id,
            grain_dir: None,
            allow_rotate: true,
            margin: 0.0,
            kerf: 0.0,
        }],
        jobs: vec![NestJob {
            id: job_id,
            sheet_defs: vec![SheetDef {
                id: Uuid::new_v4(),
                material_id,
                width: 100.0,
                height: 100.0,
                quantity: 1,
            }],
            parts_ref: vec![PartRef {
                part_id,
                quantity_override: None,
            }],
            constraints: NestConstraints {
                global_margin: 0.0,
                global_kerf: 0.0,
                allow_rotate_default: true,
                no_go_zones: vec![],
                grain_policy: GrainPolicy::Ignore,
            },
            objective: NestObjective {
                w_utilization: 1.0,
                w_sheet_count: 1.0,
                w_cut_count: 0.01,
            },
            seed: 42,
            result: None,
            trace: None,
        }],
        materials: vec![Material {
            id: material_id,
            name: "M".into(),
            category: MaterialCategory::Other,
            thickness_mm: None,
            sheet_default: None,
            notes: String::new(),
        }],
        settings: ProjectSettings::default(),
    };
    (doc, job_id)
}

#[test]
fn run_nesting_undo_redo_roundtrip() {
    let (mut doc, job_id) = mk_doc();
    let before = serde_json::to_value(&doc).unwrap();
    let mut cmd = RunNestingCommand::new();
    cmd.begin(&CommandContext::default()).unwrap();
    cmd.update(RunNestingInput {
        job_id,
        eps: EpsilonPolicy::default(),
        limits: RunLimits {
            time_limit_ms: 10,
            iteration_limit: 3,
        },
        doc_snapshot: doc.clone(),
    })
    .unwrap();
    let delta = cmd.commit().unwrap();
    delta.apply(&mut doc).unwrap();
    let mut h = History::new();
    h.push(delta);
    assert!(doc.jobs[0].result.is_some());
    h.undo(&mut doc).unwrap();
    assert_eq!(serde_json::to_value(&doc).unwrap(), before);
    h.redo(&mut doc).unwrap();
    assert!(doc.jobs[0].result.is_some());
}
