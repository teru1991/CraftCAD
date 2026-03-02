use drawing_model::{
    AnnotationEntity, AnnotationKind, AnnotationPayload, AnnotationType, DimensionEntity,
    DimensionKind, DimensionOverrides, DimensionType, DrawingDoc, GeometryRef, PlacementHint,
    PlacementSide, RefKind, RefSpace,
};
use drawing_style::ProjectMeta;
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

fn g_point(id: &str) -> GeometryRef {
    GeometryRef {
        space: RefSpace::Sketch,
        kind: RefKind::Point,
        stable_id: id.to_string(),
    }
}

fn g_segment(id: &str) -> GeometryRef {
    GeometryRef {
        space: RefSpace::Sketch,
        kind: RefKind::Segment,
        stable_id: id.to_string(),
    }
}

fn mk_payload(v: serde_json::Value) -> AnnotationPayload {
    serde_json::from_value(v).expect("invalid payload")
}

#[test]
fn export_svg_with_dimensions_and_annotations() {
    let root = repo_root();

    let mut doc = DrawingDoc::new_minimal("DRW_000001");
    doc.dimensions.push(DimensionEntity {
        id: "DIM_000001".to_string(),
        kind: DimensionKind {
            ty: DimensionType::LinearSerial,
        },
        ref_geometry: vec![g_segment("SEG_TOP")],
        placement_hint: PlacementHint {
            side: PlacementSide::Top,
            offset_level: 0,
            manual_text_pos_mm: None,
        },
        overrides: DimensionOverrides::default(),
    });

    doc.annotations.push(AnnotationEntity {
        id: "ANN_000001".to_string(),
        kind: AnnotationKind {
            ty: AnnotationType::HoleCallout,
        },
        ref_geometry: vec![g_point("P_ORIGIN")],
        placement_hint: PlacementHint::default(),
        payload: mk_payload(json!({"type":"hole","hole_diameter_mm":10.0,"hole_count":1})),
    });
    doc.annotations.push(AnnotationEntity {
        id: "ANN_000002".to_string(),
        kind: AnnotationKind {
            ty: AnnotationType::ChamferCallout,
        },
        ref_geometry: vec![g_point("P_BOARD_A")],
        placement_hint: PlacementHint::default(),
        payload: mk_payload(json!({"type":"chamfer","chamfer_type":"C","chamfer_value_mm":0.5})),
    });

    let req = ExportRequest {
        style_preset_id: doc.style_preset_id.clone(),
        sheet_template_id: doc.sheet_template_id.clone(),
        print_preset_id: doc.print_preset_id.clone(),
        meta: ProjectMeta {
            project_title: "Demo".to_string(),
            drawing_title: "Board+Hole+Chamfer".to_string(),
            scale: "1:1".to_string(),
            unit: "mm".to_string(),
            date: "2026-03-02".to_string(),
            author: "CraftCAD".to_string(),
            revision: "A".to_string(),
            schema_version: "doc_v10".to_string(),
            app_version: "0.1.0".to_string(),
        },
    };

    let svg = DrawingExporter::export_svg(&root, Some(&doc), &req).expect("export failed");
    assert!(svg.contains("<polyline"));
    assert!(svg.contains("<circle"));
    assert!(svg.contains("<text"));
    assert!(svg.contains("⌀"));
}
