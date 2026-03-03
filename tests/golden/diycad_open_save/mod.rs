use diycad_format::{
    open_package, save_package, Document, Entrypoints, Manifest, NestJob, OpenOptions, OpenResult,
    SaveOptions, Unit,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;

#[derive(Debug, Serialize, Deserialize, Default)]
struct GoldenSignatures {
    open_save_open_ok: String,
    broken_part_salvage: String,
    missing_manifest_salvage: String,
}

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

fn mutate_zip_entries<F>(src: &Path, dst: &Path, mutator: F)
where
    F: FnOnce(&mut BTreeMap<String, Vec<u8>>),
{
    let src_file = fs::File::open(src).expect("open src");
    let mut zr = zip::ZipArchive::new(src_file).expect("open zip");
    let mut entries = BTreeMap::<String, Vec<u8>>::new();
    for i in 0..zr.len() {
        let mut f = zr.by_index(i).expect("entry");
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).expect("read entry");
        entries.insert(f.name().to_string(), bytes);
    }

    mutator(&mut entries);

    let dst_file = fs::File::create(dst).expect("create dst");
    let mut zw = zip::ZipWriter::new(dst_file);
    let opt = SimpleFileOptions::default();
    for (name, bytes) in entries {
        zw.start_file(name, opt).expect("start file");
        zw.write_all(&bytes).expect("write file");
    }
    zw.finish().expect("finish zip");
}

#[test]
fn golden_open_save_and_salvage_signatures() {
    let td = tempdir().expect("tmp");
    let ok_path = td.path().join("ok_minimal.diycad");
    save_package(
        &ok_path,
        SaveOptions::default(),
        minimal_manifest(1),
        minimal_doc(),
        vec![],
        Vec::<NestJob>::new(),
        vec![],
    )
    .expect("save ok");

    let open1 = open_package(&ok_path, OpenOptions::default()).expect("open1");
    let saved_path = td.path().join("saved.diycad");
    save_package(
        &saved_path,
        SaveOptions::default(),
        open1.manifest.clone().expect("manifest"),
        open1.document.clone(),
        open1.parts_loaded.clone(),
        open1.nest_jobs_loaded.clone(),
        vec![],
    )
    .expect("save reopened");
    let open2 = open_package(&saved_path, OpenOptions::default()).expect("open2");
    let ok_sig = signature_hash(&open2);

    let broken_path = td.path().join("broken_part.diycad");
    mutate_zip_entries(&ok_path, &broken_path, |entries| {
        entries.insert(
            "parts/broken.json".to_string(),
            b"{ this is not json".to_vec(),
        );
    });
    let broken = open_package(
        &broken_path,
        OpenOptions {
            allow_salvage: true,
            ..OpenOptions::default()
        },
    )
    .expect("open broken");
    assert!(broken.read_only, "broken part should force read_only");
    assert!(
        !broken.parts_failed.is_empty(),
        "broken part should be reported"
    );
    let broken_sig = signature_hash(&broken);

    let missing_manifest_path = td.path().join("missing_manifest.diycad");
    mutate_zip_entries(&ok_path, &missing_manifest_path, |entries| {
        entries.remove("manifest.json");
    });
    let missing = open_package(&missing_manifest_path, OpenOptions::default())
        .expect("open missing manifest");
    assert!(
        missing.manifest.is_none(),
        "manifest should be None when missing"
    );
    assert!(
        !missing.warnings.is_empty(),
        "missing manifest should emit warnings"
    );
    let missing_sig = signature_hash(&missing);

    let expected_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tests/golden/diycad_open_save/golden_signatures.json");
    let current = GoldenSignatures {
        open_save_open_ok: ok_sig,
        broken_part_salvage: broken_sig,
        missing_manifest_salvage: missing_sig,
    };

    let accept = std::env::var("GOLDEN_ACCEPT").ok().as_deref() == Some("1");
    if accept {
        fs::write(
            expected_path,
            serde_json::to_vec_pretty(&current).expect("serialize golden"),
        )
        .expect("write golden signatures");
        return;
    }

    let expected: GoldenSignatures = serde_json::from_slice(
        &fs::read(expected_path).expect("missing golden_signatures.json; run with GOLDEN_ACCEPT=1"),
    )
    .expect("invalid golden signatures json");

    assert_eq!(expected.open_save_open_ok, current.open_save_open_ok, "open-save-open signature changed; run GOLDEN_ACCEPT=1 cargo test -q --test diycad_open_save");
    assert_eq!(
        expected.broken_part_salvage, current.broken_part_salvage,
        "broken_part signature changed; run GOLDEN_ACCEPT=1 cargo test -q --test diycad_open_save"
    );
    assert_eq!(expected.missing_manifest_salvage, current.missing_manifest_salvage, "missing_manifest signature changed; run GOLDEN_ACCEPT=1 cargo test -q --test diycad_open_save");
}
