use craftcad_io::options::{ExportOptions, ImportOptions};
use craftcad_io_bridge::default_engine;
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

fn canonicalize_json_output(s: &str) -> String {
    let mut v: serde_json::Value = serde_json::from_str(s).expect("valid json output");
    if let Some(obj) = v.as_object_mut() {
        if let Some(meta) = obj.get_mut("metadata").and_then(|m| m.as_object_mut()) {
            meta.insert("unit_guess".to_string(), serde_json::Value::Null);
        }
        if let Some(ents) = obj.get_mut("entities").and_then(|e| e.as_array_mut()) {
            for (idx, e) in ents.iter_mut().enumerate() {
                if let Some(o) = e.as_object_mut() {
                    if let Some(t) = o.get_mut("type").and_then(|t| t.as_str()) {
                        if t == "path" {
                            o.insert(
                                "id".to_string(),
                                serde_json::Value::String(format!("path_{}", idx)),
                            );
                            if let Some(stroke) =
                                o.get_mut("stroke").and_then(|s| s.as_object_mut())
                            {
                                stroke.insert(
                                    "layer".to_string(),
                                    serde_json::Value::String("LAYER_0".to_string()),
                                );
                            }
                        } else if t == "text" {
                            o.insert(
                                "id".to_string(),
                                serde_json::Value::String(format!("text_{}", idx)),
                            );
                        }
                    }
                }
            }
        }
    }
    serde_json::to_string_pretty(&v).expect("serialize canonicalized json")
}

#[test]
fn e2e_json_to_dxf_to_json_is_stable() {
    let root = repo_root();
    let input_json = root.join("tests/golden/io_roundtrip/inputs/e2e/e2e_01.json");
    let exp_dxf = root.join("tests/golden/io_roundtrip/expected/e2e/dxf_out.dxf");
    let exp_json = root.join("tests/golden/io_roundtrip/expected/e2e/json_out.json");

    let eng = default_engine();
    let iopts = ImportOptions::default_for_tests();
    let eopts = ExportOptions::default_for_tests();

    let res0 = eng
        .import("json", &read_bytes(&input_json), &iopts)
        .unwrap();

    let dxf = eng.export("dxf", &res0.model, &eopts).unwrap();
    let dxf_s = String::from_utf8(dxf.bytes).unwrap();
    assert_or_accept(&exp_dxf, &dxf_s);

    let res1 = eng.import("dxf", dxf_s.as_bytes(), &iopts).unwrap();

    let json = eng.export("json", &res1.model, &eopts).unwrap();
    let json_s = String::from_utf8(json.bytes).unwrap();
    let json_canon = canonicalize_json_output(&json_s);
    assert_or_accept(&exp_json, &json_canon);

    let dxf2 = eng.export("dxf", &res0.model, &eopts).unwrap();
    assert_eq!(String::from_utf8(dxf2.bytes).unwrap(), dxf_s);

    let res1b = eng.import("dxf", dxf_s.as_bytes(), &iopts).unwrap();
    assert_eq!(
        serde_json::to_string(&res1.model).unwrap(),
        serde_json::to_string(&res1b.model).unwrap()
    );
}

#[test]
fn e2e_json_to_svg_to_json_is_stable() {
    let root = repo_root();
    let input_json = root.join("tests/golden/io_roundtrip/inputs/e2e/e2e_01.json");
    let exp_svg = root.join("tests/golden/io_roundtrip/expected/e2e/svg_out.svg");
    let exp_json = root.join("tests/golden/io_roundtrip/expected/e2e/json_out.json");

    let eng = default_engine();
    let iopts = ImportOptions::default_for_tests();
    let eopts = ExportOptions::default_for_tests();

    let res0 = eng
        .import("json", &read_bytes(&input_json), &iopts)
        .unwrap();

    let svg = eng.export("svg", &res0.model, &eopts).unwrap();
    let svg_s = String::from_utf8(svg.bytes).unwrap();
    assert_or_accept(&exp_svg, &svg_s);

    let res1 = eng.import("svg", svg_s.as_bytes(), &iopts).unwrap();
    let json = eng.export("json", &res1.model, &eopts).unwrap();
    let json_s = String::from_utf8(json.bytes).unwrap();
    let json_canon = canonicalize_json_output(&json_s);
    assert_or_accept(&exp_json, &json_canon);

    let svg2 = eng.export("svg", &res0.model, &eopts).unwrap();
    assert_eq!(String::from_utf8(svg2.bytes).unwrap(), svg_s);
}
