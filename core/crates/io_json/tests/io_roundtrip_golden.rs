use craftcad_io::options::{ExportOptions, ImportOptions};
use craftcad_io::reasons::ReasonCode;
use craftcad_io::IoEngine;
use craftcad_io_json::JsonIo;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read_bytes(path: &PathBuf) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
}

fn read_string(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
}

fn write_string(path: &PathBuf, s: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("failed to create dirs: {e}"));
    }
    fs::write(path, s).unwrap_or_else(|e| panic!("failed to write {}: {}", path.display(), e));
}

fn accept_enabled() -> bool {
    std::env::var("GOLDEN_ACCEPT").ok().as_deref() == Some("1")
}

fn assert_or_accept(path: &PathBuf, got: &str) {
    if accept_enabled() {
        write_string(path, got);
        return;
    }
    let exp = read_string(path);
    let v_exp: serde_json::Value = serde_json::from_str(&exp).expect("parse expected json");
    let v_got: serde_json::Value = serde_json::from_str(got).expect("parse got json");
    assert_eq!(v_exp, v_got, "golden mismatch: {}", path.display());
}

fn warnings_to_json(warnings: &[craftcad_io::reasons::AppError]) -> String {
    let mut arr = Vec::new();
    for w in warnings {
        arr.push(serde_json::json!({
            "reason": format!("{:?}", w.reason),
            "message": w.message,
            "hint": w.hint,
            "context": w.context,
            "is_fatal": w.is_fatal
        }));
    }
    serde_json::to_string_pretty(&arr).unwrap()
}

#[test]
fn json_roundtrip_golden() {
    let root = repo_root();
    let input_path = root.join("tests/golden/io_roundtrip/inputs/json/sample_01.json");
    let exp_model_path =
        root.join("tests/golden/io_roundtrip/expected/normalized_internal_model.json");
    let exp_warn_path = root.join("tests/golden/io_roundtrip/expected/warnings.json");
    let exp_out_path = root.join("tests/golden/io_roundtrip/expected/exported_out.json");

    let eng = IoEngine::new()
        .register_importer(Box::new(JsonIo::new()))
        .register_exporter(Box::new(JsonIo::new()));

    let bytes = read_bytes(&input_path);
    let iopts = ImportOptions::default_for_tests();
    let res = eng.import("json", &bytes, &iopts).unwrap();

    assert_or_accept(
        &exp_model_path,
        &serde_json::to_string_pretty(&res.model).unwrap(),
    );
    assert_or_accept(&exp_warn_path, &warnings_to_json(&res.warnings));

    let eopts = ExportOptions::default_for_tests();
    let out = eng.export("json", &res.model, &eopts).unwrap();
    assert_or_accept(&exp_out_path, &String::from_utf8(out.bytes).unwrap());

    assert!(!res
        .warnings
        .iter()
        .any(|w| w.is_fatal && w.reason == ReasonCode::IO_PARSE_JSON_MALFORMED));
}

#[test]
fn json_import_is_deterministic() {
    let root = repo_root();
    let input_path = root.join("tests/golden/io_roundtrip/inputs/json/sample_01.json");
    let bytes = read_bytes(&input_path);

    let eng = IoEngine::new().register_importer(Box::new(JsonIo::new()));
    let opts = ImportOptions::default_for_tests();

    let r1 = eng.import("json", &bytes, &opts).unwrap();
    let r2 = eng.import("json", &bytes, &opts).unwrap();

    let m1 = serde_json::to_string(&r1.model).unwrap();
    let m2 = serde_json::to_string(&r2.model).unwrap();
    assert_eq!(m1, m2);
}
