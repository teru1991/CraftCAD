use craftcad_serialize::*;
use diycad_geom::EpsilonPolicy;
use diycad_nesting::{run_nesting, RunLimits};
use uuid::Uuid;

fn rect(w: f64, h: f64) -> Polygon2D {
    Polygon2D {
        outer: vec![
            Vec2 { x: 0.0, y: 0.0 },
            Vec2 { x: w, y: 0.0 },
            Vec2 { x: w, y: h },
            Vec2 { x: 0.0, y: h },
        ],
        holes: vec![],
    }
}

fn base_doc(part_w: f64, part_h: f64, margin: f64, kerf: f64) -> (Document, Uuid) {
    let mat = Uuid::new_v4();
    let part = Uuid::new_v4();
    let job = Uuid::new_v4();
    (
        Document {
            schema_version: 1,
            id: Uuid::new_v4(),
            units: "mm".into(),
            layers: vec![],
            entities: vec![],
            parts: vec![Part {
                id: part,
                name: "P".into(),
                outline: rect(part_w, part_h),
                thickness: 1.0,
                quantity: 1,
                material_id: mat,
                grain_dir: None,
                allow_rotate: true,
                margin,
                kerf,
            }],
            jobs: vec![NestJob {
                id: job,
                sheet_defs: vec![SheetDef {
                    id: Uuid::new_v4(),
                    material_id: mat,
                    width: 20.0,
                    height: 20.0,
                    quantity: 1,
                }],
                parts_ref: vec![PartRef {
                    part_id: part,
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
                    w_cut_count: 0.0,
                },
                seed: 7,
                result: None,
                trace: None,
            }],
            materials: vec![Material {
                id: mat,
                name: "M".into(),
                category: MaterialCategory::Other,
                thickness_mm: None,
                sheet_default: None,
                notes: String::new(),
            }],
            settings: ProjectSettings::default(),
        },
        job,
    )
}

#[test]
fn determinism_same_seed_same_result() {
    let (doc, _) = base_doc(10.0, 10.0, 0.0, 0.0);
    let job = &doc.jobs[0];
    let a = run_nesting(
        job,
        &doc,
        &EpsilonPolicy::default(),
        RunLimits {
            time_limit_ms: 100,
            iteration_limit: 3,
        },
    )
    .unwrap()
    .0;
    let b = run_nesting(
        job,
        &doc,
        &EpsilonPolicy::default(),
        RunLimits {
            time_limit_ms: 100,
            iteration_limit: 3,
        },
    )
    .unwrap()
    .0;
    assert_eq!(
        serde_json::to_value(a).unwrap(),
        serde_json::to_value(b).unwrap()
    );
}

#[test]
fn too_large_reason() {
    let (doc, _) = base_doc(50.0, 50.0, 0.0, 0.0);
    let out = run_nesting(
        &doc.jobs[0],
        &doc,
        &EpsilonPolicy::default(),
        RunLimits {
            time_limit_ms: 10,
            iteration_limit: 1,
        },
    )
    .unwrap()
    .0;
    let reason = out.per_part_status[0].reason.as_ref().unwrap();
    assert_eq!(reason.code, "NEST_PART_TOO_LARGE_FOR_ANY_SHEET");
}

#[test]
fn margin_kerf_infeasible_reason() {
    let (mut doc, _) = base_doc(12.0, 12.0, 0.0, 0.0);
    doc.jobs[0].parts_ref[0].quantity_override = Some(2);
    let out = run_nesting(
        &doc.jobs[0],
        &doc,
        &EpsilonPolicy::default(),
        RunLimits {
            time_limit_ms: 10,
            iteration_limit: 1,
        },
    )
    .unwrap()
    .0;
    let reason = out
        .per_part_status
        .iter()
        .find_map(|s| s.reason.as_ref())
        .unwrap();
    assert_eq!(
        reason.code,
        "NEST_NO_FEASIBLE_POSITION_WITH_MARGIN_AND_KERF"
    );
}
