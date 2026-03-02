use regex::Regex;
use std::collections::HashSet;
use std::fs;

fn matrix_entries(matrix_value: &serde_json::Value) -> Vec<&serde_json::Value> {
    if let Some(entries) = matrix_value["matrix"].as_array() {
        entries.iter().collect()
    } else {
        Vec::new()
    }
}

/// Test that validates IO support_matrix.json integrity with catalog.json
#[test]
fn test_io_support_matrix_consistency() {
    // CARGO_MANIFEST_DIR = core/crates/io_support, so we go up to core and then to CraftCAD root
    let root_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // -> core/crates
        .unwrap()
        .parent() // -> core
        .unwrap()
        .parent() // -> CraftCAD
        .unwrap();

    // Load support_matrix.json
    let matrix_abs_path = root_path.join("docs/specs/io/support_matrix.json");
    assert!(
        matrix_abs_path.exists(),
        "support_matrix.json not found at {:?}",
        matrix_abs_path
    );

    let matrix_content =
        fs::read_to_string(&matrix_abs_path).expect("Failed to read support_matrix.json");
    let matrix_value: serde_json::Value =
        serde_json::from_str(&matrix_content).expect("Failed to parse support_matrix.json");

    // Load catalog.json
    let catalog_abs_path = root_path.join("docs/specs/errors/catalog.json");
    let catalog_content =
        fs::read_to_string(&catalog_abs_path).expect("Failed to read catalog.json");
    let catalog_value: serde_json::Value =
        serde_json::from_str(&catalog_content).expect("Failed to parse catalog.json");

    let mut catalog_codes_set = HashSet::new();
    if let Some(items) = catalog_value["items"].as_array() {
        for item in items {
            if let Some(code) = item["code"].as_str() {
                catalog_codes_set.insert(code.to_string());
            }
        }
    }

    let mut errors = Vec::new();
    for cell in matrix_entries(&matrix_value) {
        let format_name = cell["format"].as_str().unwrap_or("<unknown>");
        let feature_name = cell["feature"].as_str().unwrap_or("<unknown>");
        let level = cell["level"].as_str().unwrap_or("supported");
        if let Some(codes) = cell["reason_codes"].as_array() {
            for code in codes {
                if let Some(code_str) = code.as_str() {
                    if !code_str.is_empty() && !catalog_codes_set.contains(code_str) {
                        errors.push(format!(
                            "Format '{}' feature '{}' references reason_code '{}' which is NOT in catalog.json",
                            format_name, feature_name, code_str
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Format '{}' feature '{}' has non-string reason_code",
                        format_name, feature_name
                    ));
                }
            }
        }

        if level == "best_effort" && !cell["reason_codes"].is_array() {
            errors.push(format!(
                "Format '{}' feature '{}' has level='best_effort' but no reason_codes array",
                format_name, feature_name
            ));
        }
    }

    if !errors.is_empty() {
        panic!(
            "IO support matrix consistency check failed with {} errors:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
}

/// Test that all reason_codes in support_matrix are from IO domain
#[test]
fn test_io_reason_codes_domain() {
    let root_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // -> core/crates
        .unwrap()
        .parent() // -> core
        .unwrap()
        .parent() // -> CraftCAD
        .unwrap();
    let matrix_abs_path = root_path.join("docs/specs/io/support_matrix.json");
    let matrix_content =
        fs::read_to_string(&matrix_abs_path).expect("Failed to read support_matrix.json");
    let matrix_value: serde_json::Value =
        serde_json::from_str(&matrix_content).expect("Failed to parse support_matrix.json");

    let mut errors = Vec::new();
    for cell in matrix_entries(&matrix_value) {
        let format_name = cell["format"].as_str().unwrap_or("<unknown>");
        let feature_name = cell["feature"].as_str().unwrap_or("<unknown>");
        if let Some(codes) = cell["reason_codes"].as_array() {
            for code in codes {
                if let Some(code_str) = code.as_str() {
                    if !code_str.is_empty() && !code_str.starts_with("IO_") {
                        errors.push(format!(
                            "Format '{}' feature '{}' has reason_code '{}' which doesn't start with 'IO_'",
                            format_name, feature_name, code_str
                        ));
                    }
                }
            }
        }
    }

    if !errors.is_empty() {
        panic!(
            "IO reason codes domain check failed:\n{}",
            errors.join("\n")
        );
    }
}

/// Test mapping_rules.json invariants match current SSOT shape
#[test]
fn test_mapping_rules_schema_invariants() {
    let root_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // -> core/crates
        .unwrap()
        .parent() // -> core
        .unwrap()
        .parent() // -> CraftCAD
        .unwrap();

    let rules_abs_path = root_path.join("docs/specs/io/mapping_rules.json");
    assert!(
        rules_abs_path.exists(),
        "mapping_rules.json not found at {:?}",
        rules_abs_path
    );

    let rules_content =
        fs::read_to_string(&rules_abs_path).expect("Failed to read mapping_rules.json");
    let v: serde_json::Value =
        serde_json::from_str(&rules_content).expect("Failed to parse mapping_rules.json");

    assert_eq!(
        v["schema_version"].as_i64(),
        Some(1),
        "schema_version must be 1"
    );

    for key in ["layer", "linetype"] {
        let section = &v[key];
        let default = section["default"].as_str().unwrap_or("");
        let max_len = section["max_len"].as_u64().unwrap_or(0);
        let re = section["forbidden_chars_regex"].as_str().unwrap_or("");
        assert!(!default.is_empty(), "{key}.default must be non-empty");
        assert!(max_len > 0, "{key}.max_len must be > 0");
        Regex::new(re).expect(&format!("{key}.forbidden_chars_regex must be valid regex"));

        // aliases object must be present
        assert!(
            section["aliases"].is_object(),
            "{key}.aliases must be object"
        );
    }

    // units.default must be in units.supported
    let units_default = v["units"]["default"].as_str().unwrap_or("");
    assert!(!units_default.is_empty(), "units.default must be non-empty");
    let supported = v["units"]["supported"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        supported.iter().any(|x| x.as_str() == Some(units_default)),
        "units.default must be in units.supported"
    );

    // export section sanity
    assert!(v["export"].is_object(), "export must be object");
    let dp = v["export"]["decimal_places"].as_u64().unwrap_or(0);
    assert!(dp <= 10, "export.decimal_places too large");
    let locale = v["export"]["force_locale"].as_str().unwrap_or("");
    assert!(!locale.is_empty(), "export.force_locale must be non-empty");
}
