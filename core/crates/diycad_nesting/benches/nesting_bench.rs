use craftcad_serialize::{
    Document, GrainPolicy, Material, MaterialCategory, NestConstraints, NestJob, NestObjective,
    Part, PartRef, Polygon2D, ProjectSettings, SheetDef, Vec2,
};
use criterion::{criterion_group, criterion_main, Criterion};
use diycad_geom::EpsilonPolicy;
use diycad_nesting::{run_nesting, RunLimits};
use uuid::Uuid;

fn sample_doc_and_job() -> (Document, NestJob) {
    let mid = Uuid::new_v4();
    let mut parts = Vec::new();
    for i in 0..30 {
        parts.push(Part {
            id: Uuid::new_v4(),
            name: format!("P{i}"),
            outline: Polygon2D {
                outer: vec![
                    Vec2 { x: 0.0, y: 0.0 },
                    Vec2 { x: 20.0, y: 0.0 },
                    Vec2 { x: 20.0, y: 10.0 },
                    Vec2 { x: 0.0, y: 10.0 },
                ],
                holes: vec![],
            },
            thickness: 3.0,
            quantity: 1,
            material_id: mid,
            grain_dir: None,
            allow_rotate: true,
            margin: 0.0,
            kerf: 0.0,
        });
    }
    let doc = Document {
        schema_version: 1,
        id: Uuid::new_v4(),
        units: "mm".into(),
        layers: vec![],
        entities: vec![],
        parts: parts.clone(),
        jobs: vec![],
        materials: vec![Material {
            id: mid,
            name: "ply".into(),
            category: MaterialCategory::Wood,
            thickness_mm: Some(3.0),
            sheet_default: None,
            notes: "".into(),
        }],
        settings: ProjectSettings {
            bom_delimiter: None,
        },
    };
    let job = NestJob {
        id: Uuid::new_v4(),
        sheet_defs: vec![SheetDef {
            id: Uuid::new_v4(),
            material_id: mid,
            width: 200.0,
            height: 200.0,
            quantity: 5,
        }],
        parts_ref: parts
            .iter()
            .map(|p| PartRef {
                part_id: p.id,
                quantity_override: Some(1),
            })
            .collect(),
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
            w_cut_count: 1.0,
        },
        seed: 123,
        result: None,
        trace: None,
    };
    (doc, job)
}

fn bench_nesting(c: &mut Criterion) {
    let (doc, job) = sample_doc_and_job();
    c.bench_function("nesting_medium_case", |b| {
        b.iter(|| {
            run_nesting(
                &job,
                &doc,
                &EpsilonPolicy::default(),
                RunLimits {
                    time_limit_ms: 500,
                    iteration_limit: 50,
                },
            )
            .expect("nesting")
        })
    });
}

criterion_group!(benches, bench_nesting);
criterion_main!(benches);
