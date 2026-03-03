use diycad_format::{
    open_package, save_package, Document, Entrypoints, Manifest, NestJob, OpenOptions, OpenResult,
    SaveOptions, Unit,
};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

fn minimal_manifest(schema_version: i64) -> Manifest {
    Manifest {
        schema_version,
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
        name: "Minimal".to_string(),
        unit: Unit::Mm,
        entities: vec![],
        parts_index: vec![],
        nest_jobs_index: vec![],
        created_at: None,
        updated_at: None,
    }
}

fn signature_hash(r: &OpenResult) -> String {
    let mut warnings_codes: Vec<String> = r
        .warnings
        .iter()
        .map(|w| w.code.as_str().to_string())
        .collect();
    warnings_codes.sort();

    let mut failed_parts: Vec<String> = r
        .parts_failed
        .iter()
        .map(|f| format!("{}:{}", f.code.as_str(), f.path))
        .collect();
    failed_parts.sort();

    let mut failed_nest_jobs: Vec<String> = r
        .nest_jobs_failed
        .iter()
        .map(|f| format!("{}:{}", f.code.as_str(), f.path))
        .collect();
    failed_nest_jobs.sort();

    let signature = format!(
        "read_only={}\\nwarnings_codes={}\\nfailed_parts={}\\nfailed_nest_jobs={}\\ncounts=parts_loaded:{};nest_jobs_loaded:{}",
        r.read_only,
        warnings_codes.join(","),
        failed_parts.join(","),
        failed_nest_jobs.join(","),
        r.parts_loaded.len(),
        r.nest_jobs_loaded.len(),
    );

    let mut hasher = Sha256::new();
    hasher.update(signature.as_bytes());
    hex::encode(hasher.finalize())
}

#[test]
fn open_signature_is_stable_for_same_input_10_times() {
    let td = tempdir().expect("tmp");
    let p = td.path().join("ok_minimal.diycad");

    save_package(
        &p,
        SaveOptions::default(),
        minimal_manifest(1),
        minimal_doc(),
        vec![],
        Vec::<NestJob>::new(),
        vec![],
    )
    .expect("save");

    let mut expected: Option<String> = None;
    for _ in 0..10 {
        let r = open_package(&p, OpenOptions::default()).expect("open");
        let current = signature_hash(&r);
        match &expected {
            Some(prev) => assert_eq!(prev, &current, "signature hash changed across runs"),
            None => expected = Some(current),
        }
    }
}
