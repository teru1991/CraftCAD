use serde_json::json;

#[path = "../../../src/testing/golden_harness.rs"]
mod golden_harness;

use golden_harness::{
    canonical_reason_codes, compare_bytes_hash, compare_expected, compare_json_struct,
    compare_reason_codes, hash_bytes, write_failure_artifacts, ActualData, CompareKind,
    DatasetMeta, ExpectedEntry, InputRef,
};

fn repo_root() -> std::path::PathBuf {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("ssot_lint crate must be under core/crates/ssot_lint")
        .to_path_buf()
}

#[test]
fn golden_harness_smoke_json_and_reasoncodes_and_bytes() {
    let root = repo_root();
    let temp = tempfile::tempdir().unwrap();
    std::env::set_var("CRAFTCAD_FAILURE_ARTIFACTS_DIR", temp.path());

    let meta = DatasetMeta {
        id: "harness_smoke".to_string(),
        seed: 7,
        epsilon: 0.0001,
        round_step: 0.0001,
        ordering_tag: "smoke".to_string(),
        limits_ref: "default".to_string(),
        inputs: vec![InputRef {
            kind: "json".to_string(),
            path: "tests/golden/inputs/io/json/min_rect.json".to_string(),
            sha256: None,
        }],
    };

    assert_eq!(
        hash_bytes(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );

    let expected_json = temp.path().join("expected.json");
    std::fs::write(
        &expected_json,
        serde_json::to_vec_pretty(&json!({"a": 1.00004, "b": {"x": 2}})).unwrap(),
    )
    .unwrap();

    compare_json_struct(
        &root,
        &meta,
        &expected_json,
        json!({"b":{"x":2}, "a": 1.000049}),
    )
    .unwrap();

    let codes =
        canonical_reason_codes(&json!({"warnings":[{"code":"B"},{"code":"A"},{"code":"A"}]}));
    assert_eq!(codes, vec!["A".to_string(), "B".to_string()]);

    let expected_warn = temp.path().join("expected_warnings.json");
    std::fs::write(
        &expected_warn,
        serde_json::to_vec_pretty(&json!({"warnings":[{"code":"A"},{"code":"B"}]})).unwrap(),
    )
    .unwrap();
    compare_reason_codes(
        &root,
        &meta,
        &expected_warn,
        json!({"warnings":[{"code":"B"},{"code":"A"}]}),
    )
    .unwrap();

    let expected_bin = temp.path().join("expected.bin");
    std::fs::write(&expected_bin, b"xyz").unwrap();
    let mismatch = compare_bytes_hash(&root, &meta, &expected_bin, b"xyZ").unwrap_err();
    assert!(mismatch.to_string().contains("bytes_hash mismatch"));

    let expected_entry = ExpectedEntry {
        compare: CompareKind::JsonStruct,
        expected_path: expected_json.clone(),
    };
    compare_expected(
        &root,
        &meta,
        &expected_entry,
        ActualData::Json(json!({"a": 1.000049, "b": {"x": 2}})),
    )
    .unwrap();

    let out_dir = temp.path().join("harness_smoke");
    assert!(out_dir.join("meta.json").exists());
    assert!(out_dir.join("diff.txt").exists());
    assert!(out_dir.join("actual.bin").exists());
}

#[test]
fn write_failure_artifacts_creates_expected_files() {
    let root = repo_root();
    let temp = tempfile::tempdir().unwrap();
    std::env::set_var("CRAFTCAD_FAILURE_ARTIFACTS_DIR", temp.path());

    let meta = DatasetMeta {
        id: "artifact_smoke".to_string(),
        seed: 1,
        epsilon: 0.0001,
        round_step: 0.0001,
        ordering_tag: "tag".to_string(),
        limits_ref: "default".to_string(),
        inputs: vec![],
    };

    let expected_path = temp.path().join("expected.bin");
    std::fs::write(&expected_path, b"aaa").unwrap();

    let codes = vec!["R1".to_string()];
    let out_dir = write_failure_artifacts(
        &root,
        &meta,
        &expected_path,
        "actual.bin",
        b"bbb",
        "diff",
        Some(&codes),
    )
    .unwrap();

    assert!(out_dir.join("meta.json").exists());
    assert!(out_dir.join("expected.bin").exists());
    assert!(out_dir.join("actual.bin").exists());
    assert!(out_dir.join("diff.txt").exists());
    assert!(out_dir.join("warnings_codes.json").exists());
}
