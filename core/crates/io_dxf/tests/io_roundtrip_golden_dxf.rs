use craftcad_io::options::{ExportOptions, ImportOptions};
use craftcad_io::IoEngine;
use craftcad_io_dxf::DxfIo;
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
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("failed to create dirs: {}", e));
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
    assert_eq!(exp, got, "golden mismatch: {}", path.display());
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
fn dxf_roundtrip_golden() {
    let root = repo_root();
    let input_path = root.join("tests/golden/io_roundtrip/inputs/dxf/sample_01.dxf");
    let exp_model_path =
        root.join("tests/golden/io_roundtrip/expected/dxf/normalized_internal_model.json");
    let exp_warn_path = root.join("tests/golden/io_roundtrip/expected/dxf/warnings.json");
    let exp_out_path = root.join("tests/golden/io_roundtrip/expected/dxf/exported_out.dxf");

    let eng = IoEngine::new()
        .register_importer(Box::new(DxfIo::new()))
        .register_exporter(Box::new(DxfIo::new()));

    let bytes = read_bytes(&input_path);
    let iopts = ImportOptions::default_for_tests();
    let res = eng.import("dxf", &bytes, &iopts).unwrap();

    assert_or_accept(
        &exp_model_path,
        &serde_json::to_string_pretty(&res.model).unwrap(),
    );
    assert_or_accept(&exp_warn_path, &warnings_to_json(&res.warnings));

    let eopts = ExportOptions::default_for_tests();
    let out = eng.export("dxf", &res.model, &eopts).unwrap();
    assert_or_accept(&exp_out_path, &String::from_utf8(out.bytes).unwrap());

    let res2 = eng
        .import("dxf", &read_bytes(&exp_out_path), &iopts)
        .unwrap();
    let m1 = serde_json::to_string(&res.model).unwrap();
    let m2 = serde_json::to_string(&res2.model).unwrap();
    assert_eq!(m1, m2, "dxf import/export/import should be stable");
}

#[test]
fn dxf_import_is_deterministic() {
    let root = repo_root();
    let input_path = root.join("tests/golden/io_roundtrip/inputs/dxf/sample_01.dxf");
    let bytes = read_bytes(&input_path);

    let eng = IoEngine::new().register_importer(Box::new(DxfIo::new()));
    let opts = ImportOptions::default_for_tests();

    let r1 = eng.import("dxf", &bytes, &opts).unwrap();
    let r2 = eng.import("dxf", &bytes, &opts).unwrap();

    assert_eq!(
        serde_json::to_string(&r1.model).unwrap(),
        serde_json::to_string(&r2.model).unwrap()
    );
}
