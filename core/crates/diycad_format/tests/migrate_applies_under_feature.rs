#![cfg(feature = "test_latest_2")]

use diycad_format::{
    open_package, save_package, Document, Entrypoints, Manifest, NestJob, OpenOptions, SaveOptions,
    Unit,
};
use tempfile::tempdir;

#[test]
fn migration_applies_when_latest_is_2_feature() {
    let td = tempdir().unwrap();
    let p = td.path().join("v1.diycad");

    let man = Manifest {
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
    };
    let doc = Document {
        id: "d".to_string(),
        name: "n".to_string(),
        unit: Unit::Mm,
        entities: vec![],
        parts_index: vec![],
        nest_jobs_index: vec![],
        created_at: None,
        updated_at: None,
    };

    save_package(
        &p,
        SaveOptions::default(),
        man,
        doc,
        vec![],
        Vec::<NestJob>::new(),
        vec![],
    )
    .unwrap();

    let r = open_package(&p, OpenOptions::default()).unwrap();
    assert!(
        r.migrate_report.is_some(),
        "migrate_report must be present under test_latest_2"
    );
}
