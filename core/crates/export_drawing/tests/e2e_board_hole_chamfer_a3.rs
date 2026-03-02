use drawing_style::ProjectMeta;
use export_drawing::{DrawingExporter, ExportRequest};
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

#[test]
fn e2e_board_hole_chamfer_a3() {
    let req = ExportRequest {
        style_preset_id: "default_v1".to_string(),
        sheet_template_id: "a3_portrait_v1".to_string(),
        print_preset_id: "a3_default_v1".to_string(),
        meta: ProjectMeta {
            project_title: "Demo".into(),
            drawing_title: "E2E A3".into(),
            scale: "1:1".into(),
            unit: "mm".into(),
            date: "2026-03-02".into(),
            author: "CraftCAD".into(),
            revision: "A".into(),
            schema_version: "doc_v10".into(),
            app_version: "0.1.0".into(),
        },
    };

    let svg = DrawingExporter::export_svg(&repo_root(), None, &req).expect("export failed");
    assert!(svg.contains("<polyline"));
}
