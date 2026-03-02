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
            panic!(
                "failed to locate repo root containing docs/specs/drawing_style/style_ssot.json"
            );
        }
    }
}

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

#[test]
fn svg_is_deterministic_for_same_input() {
    let root = repo_root();
    let req = ExportRequest {
        style_preset_id: "default_v1".to_string(),
        sheet_template_id: "a4_portrait_v1".to_string(),
        print_preset_id: "a4_default_v1".to_string(),
        meta: ProjectMeta {
            project_title: "Demo".to_string(),
            drawing_title: "Determinism".to_string(),
            scale: "1:1".to_string(),
            unit: "mm".to_string(),
            date: "2026-03-02".to_string(),
            author: "CraftCAD".to_string(),
            revision: "A".to_string(),
            schema_version: "doc_v10".to_string(),
            app_version: "0.1.0".to_string(),
        },
    };

    let mut hashes: Vec<u64> = vec![];
    for _ in 0..10 {
        let svg = DrawingExporter::export_svg(&root, None, &req).expect("export failed");
        hashes.push(hash_str(&normalize_svg_for_golden(&svg)));
    }
    for i in 1..hashes.len() {
        assert_eq!(
            hashes[0], hashes[i],
            "hash mismatch at {}: {} vs {}",
            i, hashes[0], hashes[i]
        );
    }
}
