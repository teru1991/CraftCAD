use diycad_format::{
    open_package, save_package, Document, Entrypoints, Manifest, NestJob, OpenOptions, SaveOptions,
    Unit,
};
use tempfile::tempdir;

fn minimal_manifest() -> Manifest {
    Manifest {
        schema_version: 1,
        app_version: "0.0.0-test".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        unit: Unit::Mm,
        entrypoints: Entrypoints {
            document: "document.json".to_string(),
        },
        features: None,
        determinism_tag: None,
        content_manifest: None,
    }
}

fn minimal_doc() -> Document {
    Document {
        id: "doc1".to_string(),
        name: "Test".to_string(),
        unit: Unit::Mm,
        entities: vec![],
        parts_index: vec![],
        nest_jobs_index: vec![],
        created_at: Some("2026-01-01T00:00:00Z".to_string()),
        updated_at: Some("2026-01-01T00:00:00Z".to_string()),
    }
}

#[test]
fn open_is_deterministic_over_multiple_runs() {
    let td = tempdir().expect("tmp");
    let p = td.path().join("a.diycad");

    save_package(
        &p,
        SaveOptions::default(),
        minimal_manifest(),
        minimal_doc(),
        vec![],
        Vec::<NestJob>::new(),
        vec![],
    )
    .expect("save");

    let opt = OpenOptions::default();
    let mut sig: Option<(bool, Vec<String>, Vec<String>)> = None;

    for _ in 0..10 {
        let r = open_package(&p, opt.clone()).expect("open");
        let w: Vec<String> = r
            .warnings
            .iter()
            .map(|x| format!("{}:{:?}:{:?}", x.code.as_str(), x.path, x.kind))
            .collect();
        let f: Vec<String> = r
            .parts_failed
            .iter()
            .map(|x| format!("{}:{}", x.code.as_str(), x.path))
            .collect();
        let s = (r.read_only, w, f);
        if let Some(prev) = &sig {
            assert_eq!(prev.0, s.0);
            assert_eq!(prev.1, s.1);
            assert_eq!(prev.2, s.2);
        } else {
            sig = Some(s);
        }
    }
}
