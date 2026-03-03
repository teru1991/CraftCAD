use diycad_format::{save_package, Document, Entrypoints, Manifest, NestJob, SaveOptions, Unit};
use std::fs;
use std::process::Command;
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

#[test]
fn migrate_verify_batch_generates_summary_without_crash() {
    let td = tempdir().expect("tmp");
    let input_dir = td.path().join("input");
    let output_dir = td.path().join("output");
    let summary_path = td.path().join("summary.json");
    fs::create_dir_all(&input_dir).expect("mk input");
    fs::create_dir_all(&output_dir).expect("mk output");

    let ok_path = input_dir.join("ok.diycad");
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

    let bad_path = input_dir.join("bad.diycad");
    fs::write(&bad_path, b"not a zip").expect("write bad");

    if let Some(bin) = option_env!("CARGO_BIN_EXE_diycad-migrate") {
        let status = Command::new(bin)
            .arg("--batch")
            .arg(&input_dir)
            .arg("--output-dir")
            .arg(&output_dir)
            .arg("--json-summary")
            .arg(&summary_path)
            .status()
            .expect("spawn diycad-migrate");
        assert!(
            status.success(),
            "batch migrate should continue with bad inputs"
        );
    } else {
        let status = Command::new("cargo")
            .current_dir(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tools/migrate"),
            )
            .arg("run")
            .arg("--quiet")
            .arg("--")
            .arg("--batch")
            .arg(&input_dir)
            .arg("--output-dir")
            .arg(&output_dir)
            .arg("--json-summary")
            .arg(&summary_path)
            .status()
            .expect("run cargo migrate");
        assert!(
            status.success(),
            "cargo-run batch migrate should continue with bad inputs"
        );
    }

    let summary = fs::read_to_string(&summary_path).expect("summary exists");
    assert!(
        summary.contains("\"files\""),
        "summary should include files section"
    );
}
