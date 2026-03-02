use drawing_model::{
    AnnotationEntity, AnnotationKind, AnnotationPayload, AnnotationType, DimensionEntity,
    DimensionKind, DimensionType, DrawingDoc, GeometryRef, PlacementHint, PlacementSide, RefKind,
    RefSpace,
};
use drawing_style::{normalize_svg_for_golden, ProjectMeta};
use export_drawing::{DrawingExporter, ExportRequest};
use serde_json::json;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if p.join("docs/specs/drawing_style/style_ssot.json").exists() {
            return p;
        }
        if !p.pop() {
            panic!("failed to find repo root");
        }
    }
}

fn g(kind: RefKind, id: &str) -> GeometryRef {
    GeometryRef {
        space: RefSpace::Sketch,
        kind,
        stable_id: id.to_string(),
    }
}

fn mk_payload(v: serde_json::Value) -> AnnotationPayload {
    serde_json::from_value(v).expect("invalid payload")
}

#[test]
fn e2e_board_hole_chamfer_a4() {
    let mut doc = DrawingDoc::new_minimal("DRW_E2E_A4");
    doc.dimensions.push(DimensionEntity {
        id: "DIM_W".to_string(),
        kind: DimensionKind {
            ty: DimensionType::LinearBaseline,
        },
        ref_geometry: vec![g(RefKind::Segment, "SEG_TOP")],
        placement_hint: PlacementHint {
            side: PlacementSide::Top,
            ..Default::default()
        },
        overrides: Default::default(),
    });
    doc.dimensions.push(DimensionEntity {
        id: "DIM_DIA".to_string(),
        kind: DimensionKind {
            ty: DimensionType::Diameter,
        },
        ref_geometry: vec![g(RefKind::Circle, "HOLE_1")],
        placement_hint: PlacementHint::default(),
        overrides: Default::default(),
    });
    doc.annotations.push(AnnotationEntity {
        id: "ANN_HOLE".to_string(),
        kind: AnnotationKind {
            ty: AnnotationType::HoleCallout,
        },
        ref_geometry: vec![g(RefKind::Point, "P_ORIGIN")],
        placement_hint: PlacementHint::default(),
        payload: mk_payload(json!({"type":"hole","hole_diameter_mm":10.0,"hole_count":1})),
    });
    doc.annotations.push(AnnotationEntity {
        id: "ANN_CH".to_string(),
        kind: AnnotationKind {
            ty: AnnotationType::ChamferCallout,
        },
        ref_geometry: vec![g(RefKind::Point, "P_BOARD_A")],
        placement_hint: PlacementHint::default(),
        payload: mk_payload(json!({"type":"chamfer","chamfer_type":"C","chamfer_value_mm":1.0})),
    });

    let req = ExportRequest {
        style_preset_id: "default_v1".to_string(),
        sheet_template_id: "a4_portrait_v1".to_string(),
        print_preset_id: "a4_default_v1".to_string(),
        meta: ProjectMeta {
            project_title: "Demo".into(),
            drawing_title: "E2E A4".into(),
            scale: "1:1".into(),
            unit: "mm".into(),
            date: "2026-03-02".into(),
            author: "CraftCAD".into(),
            revision: "A".into(),
            schema_version: "doc_v10".into(),
            app_version: "0.1.0".into(),
        },
    };

    let svg = DrawingExporter::export_svg(&repo_root(), Some(&doc), &req).expect("export failed");
    let n = normalize_svg_for_golden(&svg);
    assert!(n.contains("clipPath"));
    assert!(n.contains("⌀"));
    assert!(n.contains("C1"));
}
