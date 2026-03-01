use craftcad_serialize::{validate_document_json_str, validate_manifest_json_str};

fn read_fixture(path: &str) -> String {
    std::fs::read_to_string(path).expect("read fixture")
}

#[test]
fn manifest_schema_accepts_minimal() {
    let s = r#"
    {
      "schema_version": 1,
      "app": { "name": "CraftCAD", "version": "0.1.0" },
      "created_at": "2026-01-01T00:00:00Z",
      "document_path": "data/document.json",
      "assets_path": "assets/",
      "settings_digest": "abcd"
    }
    "#;
    validate_manifest_json_str(s).expect("manifest should validate");
}

#[test]
fn document_schema_accepts_sample_fixture() {
    let s = read_fixture("../../tests/fixtures/sample_project_v1.json");
    validate_document_json_str(&s).expect("document fixture should validate");
}

#[test]
fn document_schema_rejects_missing_units() {
    let s = read_fixture("../../tests/fixtures/invalid_missing_units.json");
    let err = validate_document_json_str(&s).expect_err("should fail");
    assert_eq!(err.code, "SERIALIZE_SCHEMA_VALIDATION_FAILED");
}

#[test]
fn document_schema_rejects_bad_geom_type() {
    let s = read_fixture("../../tests/fixtures/invalid_bad_geom_type.json");
    let err = validate_document_json_str(&s).expect_err("should fail");
    assert_eq!(err.code, "SERIALIZE_SCHEMA_VALIDATION_FAILED");
}

#[test]
fn old_doc_without_materials_is_normalized() {
    let json =
        std::fs::read_to_string("../../tests/fixtures/sample_project_v1.json").expect("fixture");
    let val = craftcad_serialize::validate_document_json_str(&json).expect("valid");
    assert!(val.get("materials").is_some());
    assert_eq!(val.get("materials").unwrap().as_array().unwrap().len(), 0);
}
