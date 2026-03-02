use diycad_format::{save_package, Document, Entrypoints, Manifest, SaveOptions, Unit};
use tempfile::tempdir;

fn manifest() -> Manifest {
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

fn doc(name: &str) -> Document {
    Document {
        id: "doc1".to_string(),
        name: name.to_string(),
        unit: Unit::Mm,
        entities: vec![],
        parts_index: vec![],
        nest_jobs_index: vec![],
        created_at: None,
        updated_at: None,
    }
}

#[test]
fn atomic_save_does_not_leave_missing_target() {
    let td = tempdir().expect("tmp");
    let p = td.path().join("p.diycad");

    save_package(
        &p,
        SaveOptions::default(),
        manifest(),
        doc("A"),
        vec![],
        vec![],
        vec![],
    )
    .expect("first save");
    assert!(p.exists());

    save_package(
        &p,
        SaveOptions::default(),
        manifest(),
        doc("B"),
        vec![],
        vec![],
        vec![],
    )
    .expect("second save");
    assert!(p.exists());

    let tmp = td.path().join(".p.diycad.tmp");
    assert!(!tmp.exists(), "tmp should not remain");
}
