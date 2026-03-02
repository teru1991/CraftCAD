use drawing_style::{normalize_svg_for_golden, ProjectMeta};
use export_drawing::{DrawingExporter, ExportRequest};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

#[test]
fn deterministic_layout_output_10x() {
    let req = ExportRequest {
        style_preset_id: "default_v1".to_string(),
        sheet_template_id: "a4_portrait_v1".to_string(),
        print_preset_id: "a4_default_v1".to_string(),
        meta: ProjectMeta {
            project_title: "Demo".into(),
            drawing_title: "LayoutDet".into(),
            scale: "1:1".into(),
            unit: "mm".into(),
            date: "2026-03-02".into(),
            author: "CraftCAD".into(),
            revision: "A".into(),
            schema_version: "doc_v10".into(),
            app_version: "0.1.0".into(),
        },
    };

    let mut hs = vec![];
    for _ in 0..10 {
        let svg = DrawingExporter::export_svg(&repo_root(), None, &req).expect("export failed");
        hs.push(hash_str(&normalize_svg_for_golden(&svg)));
    }
    assert!(hs.iter().all(|h| *h == hs[0]));
}
