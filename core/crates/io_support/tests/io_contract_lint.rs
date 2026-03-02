use regex::Regex;
use std::collections::HashSet;
use std::fs;

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("io_support")
        .parent()
        .expect("crates")
        .parent()
        .expect("core")
        .to_path_buf()
}

fn load_json(path: &std::path::PathBuf) -> serde_json::Value {
    let s = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e))
}

#[test]
fn ssot_support_matrix_reason_codes_must_exist_in_catalog() {
    let root = repo_root();
    let matrix_path = root.join("docs/specs/io/support_matrix.json");
    let catalog_path = root.join("docs/specs/errors/catalog.json");

    let matrix = load_json(&matrix_path);
    let catalog = load_json(&catalog_path);

    let mut catalog_set = HashSet::new();
    if let Some(items) = catalog["items"].as_array() {
        for it in items {
            if let Some(code) = it["code"].as_str() {
                catalog_set.insert(code.to_string());
            }
        }
    }
    assert!(!catalog_set.is_empty(), "catalog.json items empty?");

    let mut missing = Vec::new();
    if let Some(entries) = matrix["matrix"].as_array() {
        for cell in entries {
            if let Some(arr) = cell["reason_codes"].as_array() {
                for c in arr {
                    if let Some(code) = c.as_str() {
                        if !code.is_empty() && !catalog_set.contains(code) {
                            missing.push(code.to_string());
                        }
                    }
                }
            }
        }
    }
    missing.sort();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "support_matrix.json references reason_codes missing in catalog.json: {:?}",
        missing
    );
}

#[test]
fn ssot_support_matrix_must_be_well_formed() {
    let root = repo_root();
    let matrix_path = root.join("docs/specs/io/support_matrix.json");
    let v = load_json(&matrix_path);

    assert_eq!(
        v["schema_version"].as_i64(),
        Some(1),
        "schema_version must be 1"
    );

    let formats: HashSet<String> = v["formats"]
        .as_array()
        .expect("formats")
        .iter()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect();
    let directions: HashSet<String> = v["directions"]
        .as_array()
        .expect("directions")
        .iter()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect();
    let levels: HashSet<String> = v["levels"]
        .as_array()
        .expect("levels")
        .iter()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect();

    assert!(formats.contains("dxf") && formats.contains("svg") && formats.contains("json"));
    assert!(directions.contains("import") && directions.contains("export"));
    assert!(
        levels.contains("supported")
            && levels.contains("best_effort")
            && levels.contains("not_supported")
    );

    let mut seen = HashSet::new();
    let mut dupes = Vec::new();

    if let Some(entries) = v["matrix"].as_array() {
        for cell in entries {
            let f = cell["format"].as_str().unwrap_or("");
            let d = cell["direction"].as_str().unwrap_or("");
            let feat = cell["feature"].as_str().unwrap_or("");
            let lvl = cell["level"].as_str().unwrap_or("");

            assert!(formats.contains(f), "unknown format in matrix: {}", f);
            assert!(directions.contains(d), "unknown direction in matrix: {}", d);
            assert!(levels.contains(lvl), "unknown level in matrix: {}", lvl);
            assert!(!feat.is_empty(), "feature must be non-empty");

            let key = format!("{}|{}|{}", f, d, feat);
            if !seen.insert(key.clone()) {
                dupes.push(key.clone());
            }

            if lvl == "best_effort" || lvl == "not_supported" {
                assert!(
                    cell["reason_codes"].is_array(),
                    "feature {} must have reason_codes array",
                    key
                );
                assert!(
                    cell["action"].is_string(),
                    "feature {} must have action string",
                    key
                );
            }
        }
    }

    dupes.sort();
    dupes.dedup();
    assert!(
        dupes.is_empty(),
        "support_matrix has duplicate entries: {:?}",
        dupes
    );
}

#[test]
fn ssot_mapping_rules_schema_invariants() {
    let root = repo_root();
    let rules_path = root.join("docs/specs/io/mapping_rules.json");
    let v = load_json(&rules_path);

    assert_eq!(
        v["schema_version"].as_i64(),
        Some(1),
        "schema_version must be 1"
    );

    for key in ["layer", "linetype"] {
        let sec = &v[key];
        let default = sec["default"].as_str().unwrap_or("");
        let max_len = sec["max_len"].as_u64().unwrap_or(0);
        let re = sec["forbidden_chars_regex"].as_str().unwrap_or("");
        assert!(!default.is_empty(), "{key}.default must be non-empty");
        assert!(max_len > 0, "{key}.max_len must be > 0");
        Regex::new(re)
            .unwrap_or_else(|_| panic!("{key}.forbidden_chars_regex must be valid regex"));
        assert!(sec["aliases"].is_object(), "{key}.aliases must be object");
        assert!(
            sec["normalize"].is_object(),
            "{key}.normalize must be object"
        );
    }

    let ud = v["units"]["default"].as_str().unwrap_or("");
    let supported = v["units"]["supported"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        supported.iter().any(|x| x.as_str() == Some(ud)),
        "units.default must be in units.supported"
    );

    let dp = v["export"]["decimal_places"].as_u64().unwrap_or(0);
    assert!(dp <= 10, "export.decimal_places too large");
    let locale = v["export"]["force_locale"].as_str().unwrap_or("");
    assert!(!locale.is_empty(), "export.force_locale must be non-empty");
}
