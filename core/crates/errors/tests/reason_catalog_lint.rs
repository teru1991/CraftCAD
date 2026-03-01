use std::collections::{HashMap, HashSet};
use std::fs;

/// Test that validates the integrity of reason codes between reason_codes.json and catalog.json
#[test]
fn test_reason_codes_catalog_consistency() {
    // Load reason_codes.json (simple list)
    // CARGO_MANIFEST_DIR = core/crates/errors, so we go up to core and then to errors
    let abs_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()           // -> core/crates
        .unwrap()
        .parent()           // -> core
        .unwrap()
        .join("errors/reason_codes.json");

    assert!(
        abs_path.exists(),
        "reason_codes.json not found at {:?}",
        abs_path
    );

    let reason_codes_content = fs::read_to_string(&abs_path)
        .expect("Failed to read reason_codes.json");
    let reason_codes_value: serde_json::Value =
        serde_json::from_str(&reason_codes_content)
            .expect("Failed to parse reason_codes.json");

    let codes_array = reason_codes_value["codes"]
        .as_array()
        .expect("'codes' field must be an array");

    let mut reason_codes_set = HashSet::new();
    for code in codes_array.iter() {
        let code_str = code
            .as_str()
            .expect("Each code must be a string");
        reason_codes_set.insert(code_str.to_string());
    }

    // Load catalog.json
    let catalog_abs_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()           // -> core/crates
        .unwrap()
        .parent()           // -> core
        .unwrap()
        .parent()           // -> CraftCAD
        .unwrap()
        .join("docs/specs/errors/catalog.json");
    assert!(
        catalog_abs_path.exists(),
        "catalog.json not found at {:?}",
        catalog_abs_path
    );

    let catalog_content = fs::read_to_string(&catalog_abs_path)
        .expect("Failed to read catalog.json");
    let catalog_value: serde_json::Value =
        serde_json::from_str(&catalog_content)
            .expect("Failed to parse catalog.json");

    let items = catalog_value["items"]
        .as_array()
        .expect("'items' field must be an array");

    let mut catalog_codes_set = HashSet::new();
    let mut errors = Vec::new();

    for item in items.iter() {
        let code = item["code"]
            .as_str()
            .expect("Each item must have a 'code' field");
        catalog_codes_set.insert(code.to_string());

        // Check that 'domain' field exists
        if item["domain"].is_null() {
            errors.push(format!(
                "Code '{}' is missing 'domain' field",
                code
            ));
        } else {
            let domain = item["domain"].as_str().expect("domain must be string");
            // Check domain matches code prefix (e.g., domain=IO → code starts with IO_)
            if !code.starts_with(&format!("{}_", domain)) {
                errors.push(format!(
                    "Code '{}' has domain '{}' but code doesn't start with '{}_'",
                    code, domain, domain
                ));
            }
        }

        // Check that 'severity' field exists
        if item["severity"].is_null() {
            errors.push(format!(
                "Code '{}' is missing 'severity' field",
                code
            ));
        }

        // Check that 'user_actions' exists and is not empty
        if item["user_actions"].is_null() {
            errors.push(format!(
                "Code '{}' is missing 'user_actions' field",
                code
            ));
        } else if let Some(actions) = item["user_actions"].as_array() {
            if actions.is_empty() {
                errors.push(format!(
                    "Code '{}' has empty 'user_actions' array",
                    code
                ));
            } else {
                // Check that no action is empty string or too short
                for (idx, action) in actions.iter().enumerate() {
                    if let Some(action_str) = action.as_str() {
                        if action_str.is_empty() {
                            errors.push(format!(
                                "Code '{}' has empty user_action at index {}",
                                code, idx
                            ));
                        } else if action_str.len() < 5 {
                            errors.push(format!(
                                "Code '{}' has very short user_action at index {} (len={}): '{}'",
                                code, idx, action_str.len(), action_str
                            ));
                        }
                    } else {
                        errors.push(format!(
                            "Code '{}' has non-string user_action at index {}",
                            code, idx
                        ));
                    }
                }
            }
        }

        // Check that 'doc_link' exists and only references docs/ (no ..)
        if item["doc_link"].is_null() {
            errors.push(format!(
                "Code '{}' is missing 'doc_link' field",
                code
            ));
        } else if let Some(doc_link) = item["doc_link"].as_str() {
            if doc_link.contains("..") {
                errors.push(format!(
                    "Code '{}' has doc_link with '..' (path traversal): '{}'",
                    code, doc_link
                ));
            }
            if !doc_link.starts_with("docs/") {
                errors.push(format!(
                    "Code '{}' has doc_link not in docs/: '{}'",
                    code, doc_link
                ));
            }
        }
    }

    // Check for codes in reason_codes.json that are NOT in catalog.json
    for code in reason_codes_set.iter() {
        if !catalog_codes_set.contains(code) {
            errors.push(format!(
                "Code '{}' is in reason_codes.json but NOT in catalog.json",
                code
            ));
        }
    }

    // Check for codes in catalog.json that are NOT in reason_codes.json
    for code in catalog_codes_set.iter() {
        if !reason_codes_set.contains(code) {
            errors.push(format!(
                "Code '{}' is in catalog.json but NOT in reason_codes.json",
                code
            ));
        }
    }

    if !errors.is_empty() {
        panic!(
            "ReasonCode catalog consistency check failed with {} errors:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
}

/// Test that validates domain/code prefix consistency
#[test]
fn test_domain_code_prefix_consistency() {
    let catalog_abs_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()           // -> core/crates
        .unwrap()
        .parent()           // -> core
        .unwrap()
        .parent()           // -> CraftCAD
        .unwrap()
        .join("docs/specs/errors/catalog.json");
    let catalog_content = fs::read_to_string(&catalog_abs_path)
        .expect("Failed to read catalog.json");
    let catalog_value: serde_json::Value =
        serde_json::from_str(&catalog_content)
            .expect("Failed to parse catalog.json");

    let items = catalog_value["items"]
        .as_array()
        .expect("'items' field must be an array");

    let mut errors = Vec::new();
    let mut domain_code_map: HashMap<String, Vec<String>> = HashMap::new();

    for item in items.iter() {
        let code = item["code"]
            .as_str()
            .expect("code field is required");
        let domain = item["domain"]
            .as_str()
            .expect("domain field is required");

        domain_code_map
            .entry(domain.to_string())
            .or_insert_with(Vec::new)
            .push(code.to_string());

        // Verify code starts with domain_
        if !code.starts_with(&format!("{}_", domain)) {
            errors.push(format!(
                "Domain/code mismatch: '{}' has domain '{}' but doesn't start with '{}_'",
                code, domain, domain
            ));
        }
    }

    if !errors.is_empty() {
        panic!(
            "Domain/code prefix consistency check failed:\n{}",
            errors.join("\n")
        );
    }

    println!("Domain/code mapping validated:");
    for (domain, codes) in domain_code_map.iter() {
        println!("  {}: {} codes", domain, codes.len());
    }
}

/// Test that severity=FATAL codes have valid user_actions (non-crash policy)
#[test]
fn test_fatal_severity_non_crash_policy() {
    let catalog_abs_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()           // -> core/crates
        .unwrap()
        .parent()           // -> core
        .unwrap()
        .parent()           // -> CraftCAD
        .unwrap()
        .join("docs/specs/errors/catalog.json");
    let catalog_content = fs::read_to_string(&catalog_abs_path)
        .expect("Failed to read catalog.json");
    let catalog_value: serde_json::Value =
        serde_json::from_str(&catalog_content)
            .expect("Failed to parse catalog.json");

    let items = catalog_value["items"]
        .as_array()
        .expect("'items' field must be an array");

    let mut errors = Vec::new();

    for item in items.iter() {
        let code = item["code"]
            .as_str()
            .expect("code field is required");
        let severity = item["severity"]
            .as_str()
            .expect("severity field is required");

        // Even FATAL codes should have user_actions (non-crash policy)
        if severity == "FATAL" {
            if let Some(actions) = item["user_actions"].as_array() {
                if actions.is_empty() {
                    errors.push(format!(
                        "FATAL code '{}' has empty user_actions (violates non-crash policy)",
                        code
                    ));
                }
            } else {
                errors.push(format!(
                    "FATAL code '{}' is missing user_actions (violates non-crash policy)",
                    code
                ));
            }
        }
    }

    if !errors.is_empty() {
        panic!(
            "Fatal severity non-crash policy check failed:\n{}",
            errors.join("\n")
        );
    }
}

