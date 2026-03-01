use craftcad_bom::{generate_bom, write_bom_csv, CsvOptions, RoundingPolicy, UnitPolicy};
use craftcad_export::{
    compute_tiled_layout, export_svg, export_tiled_pdf, SvgExportOptions, TiledPdfOptions,
};
use craftcad_serialize::{
    Document, GrainPolicy, Material, MaterialCategory, NestConstraints, NestJob, NestObjective,
    Part, PartRef, Polygon2D, ProjectSettings, ReasonCode, SheetDef, Vec2,
};
use diycad_geom::EpsilonPolicy;
use diycad_nesting::{run_nesting, RunLimits};
use uuid::Uuid;

fn rect_part(id: Uuid, name: &str, w: f64, h: f64, mid: Uuid) -> Part {
    Part {
        id,
        name: name.into(),
        outline: Polygon2D {
            outer: vec![
                Vec2 { x: 0.0, y: 0.0 },
                Vec2 { x: w, y: 0.0 },
                Vec2 { x: w, y: h },
                Vec2 { x: 0.0, y: h },
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
    }
}

#[test]
fn e2e_create_part_nest_export() {
    let golden_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/golden/e2e/expected_pipeline.json");
    let golden: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(golden_path).expect("golden e2e"))
            .expect("golden json");
    let mid = Uuid::new_v4();
    let p1 = rect_part(Uuid::new_v4(), "P1", 40.0, 20.0, mid);
    let p2 = rect_part(Uuid::new_v4(), "P2", 20.0, 20.0, mid);
    let mut doc = Document {
        schema_version: 1,
        id: Uuid::new_v4(),
        units: "mm".into(),
        layers: vec![],
        entities: vec![],
        parts: vec![p1.clone(), p2.clone()],
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
            width: 100.0,
            height: 100.0,
            quantity: 1,
        }],
        parts_ref: vec![
            PartRef {
                part_id: p1.id,
                quantity_override: Some(1),
            },
            PartRef {
                part_id: p2.id,
                quantity_override: Some(1),
            },
        ],
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
        seed: 42,
        result: None,
        trace: None,
    };

    let (result, trace) = run_nesting(
        &job,
        &doc,
        &EpsilonPolicy::default(),
        RunLimits {
            time_limit_ms: 500,
            iteration_limit: 20,
        },
    )
    .unwrap_or_else(|e| {
        panic!(
            "e2e nesting failed code={} seed={} eps={:?}",
            e.code,
            job.seed,
            EpsilonPolicy::default()
        )
    });

    assert!(result.placements.len() >= golden["placed_min"].as_u64().unwrap() as usize);
    assert!(trace.iterations > 0);

    let mut job2 = job.clone();
    job2.result = Some(result.clone());
    job2.trace = Some(trace);
    doc.jobs = vec![job2];

    let bom = generate_bom(&doc, UnitPolicy, RoundingPolicy).expect("bom");
    assert_eq!(
        bom.rows.len(),
        golden["bom_rows"].as_u64().unwrap() as usize
    );
    let csv = write_bom_csv(&bom, CsvOptions { delimiter: ',' }).expect("csv");
    assert!(!csv.is_empty());

    let pdf = export_tiled_pdf(&doc, &TiledPdfOptions::default()).expect("pdf");
    assert!(!pdf.is_empty());
    let layout = compute_tiled_layout(&doc, &TiledPdfOptions::default()).expect("layout");
    assert!(layout.page_count >= 1);

    let svg = export_svg(&doc, &SvgExportOptions::default()).expect("svg");
    assert!(svg.contains(golden["svg_contains"].as_str().unwrap()));
    assert!(svg.contains("entity") || svg.contains("part"));

    let err = run_nesting(
        &NestJob {
            parts_ref: vec![PartRef {
                part_id: Uuid::new_v4(),
                quantity_override: Some(1),
            }],
            ..job
        },
        &doc,
        &EpsilonPolicy::default(),
        RunLimits::default(),
    )
    .expect_err("should fail with unknown part ref");
    assert_eq!(err.code, ReasonCode::ModelReferenceNotFound.as_str());
}
