use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn reason_catalog_valid_and_links_exist() {
    let schema_raw = std::fs::read_to_string("../../docs/specs/errors/catalog.schema.json")
        .expect("schema read");
    let catalog_raw =
        std::fs::read_to_string("../../docs/specs/errors/catalog.json").expect("catalog read");

    let schema: serde_json::Value = serde_json::from_str(&schema_raw).expect("schema json");
    let catalog: serde_json::Value = serde_json::from_str(&catalog_raw).expect("catalog json");

    let compiled = jsonschema::JSONSchema::compile(&schema).expect("compile schema");
    let result = compiled.validate(&catalog);
    if let Err(errors) = result {
        let issues: Vec<String> = errors.map(|e| e.to_string()).collect();
        panic!("catalog validation failed: {}", issues.join("; "));
    }

    let mut uniq = BTreeSet::new();
    for item in catalog["items"].as_array().expect("items array") {
        let code = item["code"].as_str().expect("code");
        assert!(uniq.insert(code.to_string()), "duplicate code: {code}");

        let link = item["doc_link"].as_str().expect("doc_link");
        assert!(
            Path::new("../..").join(link).exists(),
            "missing doc_link target: {link}"
        );
    }
}

#[test]
fn io_support_matrix_is_machine_readable() {
    let raw =
        std::fs::read_to_string("../../docs/specs/io/support_matrix.json").expect("support matrix");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("support matrix json");
    assert!(value["formats"].is_object());
}

#[test]
fn dataset_manifest_references_existing_files() {
    let raw =
        std::fs::read_to_string("../../tests/datasets/manifest.json").expect("dataset manifest");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("dataset manifest json");
    for ds in value["datasets"].as_array().expect("datasets") {
        for key in ["inputs", "expected_outputs"] {
            for p in ds[key].as_array().expect("paths") {
                let rel = p.as_str().expect("path str");
                assert!(
                    Path::new("../..").join(rel).exists(),
                    "missing dataset file: {rel}"
                );
            }
        }
    }
}

#[test]
fn drawing_style_ssot_is_valid_and_named_consistently() {
    let schema_raw = std::fs::read_to_string("../../docs/specs/drawing/style_ssot.schema.json")
        .expect("style schema read");
    let style_raw =
        std::fs::read_to_string("../../docs/specs/drawing/style_ssot.json").expect("style read");

    let schema: serde_json::Value = serde_json::from_str(&schema_raw).expect("style schema json");
    let style: serde_json::Value = serde_json::from_str(&style_raw).expect("style json");

    let compiled = jsonschema::JSONSchema::compile(&schema).expect("compile style schema");
    if let Err(errors) = compiled.validate(&style) {
        let issues: Vec<String> = errors.map(|e| e.to_string()).collect();
        panic!("style_ssot validation failed: {}", issues.join("; "));
    }

    let mut style_names = BTreeSet::new();
    for line_style in style["line_styles"].as_array().expect("line_styles") {
        let name = line_style["name"].as_str().expect("line_style.name");
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "invalid line style name: {name}"
        );
        assert!(
            style_names.insert(name.to_string()),
            "duplicate line style: {name}"
        );
    }

    let weights = style["line_weights"].as_object().expect("line_weights");
    let mut weight_names = BTreeSet::new();
    for name in weights.keys() {
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "invalid line weight name: {name}"
        );
        assert!(
            weight_names.insert(name.to_string()),
            "duplicate line weight: {name}"
        );
    }
}
