use drawing_style::{normalize_svg_for_golden, ProjectMeta};
use export_drawing::{DrawingExporter, ExportRequest};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if p.join("docs/specs/drawing_style/style_ssot.json").exists() {
            return p;
        }
        if !p.pop() {
            panic!(
                "failed to locate repo root containing docs/specs/drawing_style/style_ssot.json"
            );
        }
    }
}

#[test]
fn export_svg_a4_sheet_only() {
    let root = repo_root();
    let req = ExportRequest {
        style_preset_id: "default_v1".to_string(),
        sheet_template_id: "a4_portrait_v1".to_string(),
        print_preset_id: "a4_default_v1".to_string(),
        meta: ProjectMeta {
            project_title: "Demo".to_string(),
            drawing_title: "Board + Hole + Chamfer".to_string(),
            scale: "1:1".to_string(),
            unit: "mm".to_string(),
            date: "2026-03-02".to_string(),
            author: "CraftCAD".to_string(),
            revision: "A".to_string(),
            schema_version: "doc_v10".to_string(),
            app_version: "0.1.0".to_string(),
        },
    };

    let svg = DrawingExporter::export_svg(&root, None, &req).expect("export failed");
    let normalized = normalize_svg_for_golden(&svg);
    assert!(normalized.contains(r#"<svg "#));
    assert!(
        normalized.contains(r#"width="210.0000mm""#) || normalized.contains(r#"width="210mm""#)
    );
    assert!(!normalized.contains("SHEET_BORDER"));
    assert!(normalized.contains("<rect"));
    assert!(normalized.contains("<text"));
}
