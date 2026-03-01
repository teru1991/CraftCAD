use std::collections::HashSet;
use std::fs;

/// Test that validates IO support_matrix.json integrity with catalog.json
#[test]
fn test_io_support_matrix_consistency() {
    // CARGO_MANIFEST_DIR = core/crates/io_support, so we go up to core and then to CraftCAD root
    let root_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()           // -> core/crates
        .unwrap()
        .parent()           // -> core
        .unwrap()
        .parent()           // -> CraftCAD
        .unwrap();

    // Load support_matrix.json
    let matrix_abs_path = root_path.join("docs/specs/io/support_matrix.json");
    assert!(
        matrix_abs_path.exists(),
        "support_matrix.json not found at {:?}",
        matrix_abs_path
    );

    let matrix_content = fs::read_to_string(&matrix_abs_path)
        .expect("Failed to read support_matrix.json");
    let matrix_value: serde_json::Value =
        serde_json::from_str(&matrix_content)
            .expect("Failed to parse support_matrix.json");

    // Load catalog.json
    let catalog_abs_path = root_path.join("docs/specs/errors/catalog.json");
    let catalog_content = fs::read_to_string(&catalog_abs_path)
        .expect("Failed to read catalog.json");
    let catalog_value: serde_json::Value =
        serde_json::from_str(&catalog_content)
            .expect("Failed to parse catalog.json");

    let mut catalog_codes_set = HashSet::new();
    if let Some(items) = catalog_value["items"].as_array() {
        for item in items {
            if let Some(code) = item["code"].as_str() {
                catalog_codes_set.insert(code.to_string());
            }
        }
    }

    let mut errors = Vec::new();
    let formats = matrix_value["formats"]
        .as_object()
        .expect("'formats' field must be an object");

    // Validate all reason_codes referenced in support_matrix
    for (format_name, features) in formats.iter() {
        if let Some(features_obj) = features.as_object() {
            for (feature_name, cell) in features_obj.iter() {
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

                // Check that if level is best_effort, reason_codes must exist and not be empty
                if let Some(level) = cell["import"].as_str() {
                    if level == "best_effort" {
                        if let Some(codes) = cell["reason_codes"].as_array() {
                            if codes.is_empty() {
                                errors.push(format!(
                                    "Format '{}' feature '{}' has import='best_effort' but reason_codes is empty",
                                    format_name, feature_name
                                ));
                            }
                        }
                    }
                }

                if let Some(level) = cell["export"].as_str() {
                    if level == "best_effort" {
                        if let Some(codes) = cell["reason_codes"].as_array() {
                            if codes.is_empty() {
                                errors.push(format!(
                                    "Format '{}' feature '{}' has export='best_effort' but reason_codes is empty",
                                    format_name, feature_name
                                ));
                            }
                        }
                    }
                }
            }
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
        .parent()           // -> core/crates
        .unwrap()
        .parent()           // -> core
        .unwrap()
        .parent()           // -> CraftCAD
        .unwrap();
    let matrix_abs_path = root_path.join("docs/specs/io/support_matrix.json");
    let matrix_content = fs::read_to_string(&matrix_abs_path)
        .expect("Failed to read support_matrix.json");
    let matrix_value: serde_json::Value =
        serde_json::from_str(&matrix_content)
            .expect("Failed to parse support_matrix.json");

    let mut errors = Vec::new();
    let formats = matrix_value["formats"]
        .as_object()
        .expect("'formats' field must be an object");

    for (format_name, features) in formats.iter() {
        if let Some(features_obj) = features.as_object() {
            for (feature_name, cell) in features_obj.iter() {
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
        }
    }

    if !errors.is_empty() {
        panic!(
            "IO reason codes domain check failed:\n{}",
            errors.join("\n")
        );
    }
}

/// Test mapping_rules.json references valid reason_codes
#[test]
fn test_mapping_rules_reason_codes() {
    let root_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()           // -> core/crates
        .unwrap()
        .parent()           // -> core
        .unwrap()
        .parent()           // -> CraftCAD
        .unwrap();

    let rules_abs_path = root_path.join("docs/specs/io/mapping_rules.json");
    assert!(
        rules_abs_path.exists(),
        "mapping_rules.json not found at {:?}",
        rules_abs_path
    );

    let rules_content = fs::read_to_string(&rules_abs_path)
        .expect("Failed to read mapping_rules.json");
    let rules_value: serde_json::Value =
        serde_json::from_str(&rules_content)
            .expect("Failed to parse mapping_rules.json");

    // Load catalog.json
    let catalog_abs_path = root_path.join("docs/specs/errors/catalog.json");
    let catalog_content = fs::read_to_string(&catalog_abs_path)
        .expect("Failed to read catalog.json");
    let catalog_value: serde_json::Value =
        serde_json::from_str(&catalog_content)
            .expect("Failed to parse catalog.json");

    let mut catalog_codes_set = HashSet::new();
    if let Some(items) = catalog_value["items"].as_array() {
        for item in items {
            if let Some(code) = item["code"].as_str() {
                catalog_codes_set.insert(code.to_string());
            }
        }
    }

    let mut errors = Vec::new();

    // Check unit_rules.reason_code_on_assume
    if let Some(code) = rules_value["unit_rules"]["reason_code_on_assume"].as_str() {
        if !code.is_empty() && !catalog_codes_set.contains(code) {
            errors.push(format!(
                "unit_rules.reason_code_on_assume='{}' is NOT in catalog.json",
                code
            ));
        }
    }

    if !errors.is_empty() {
        panic!(
            "Mapping rules reason codes check failed:\n{}",
            errors.join("\n")
        );
    }
}

