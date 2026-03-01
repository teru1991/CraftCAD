#[test]
fn schema_ids_are_unique_and_validated() {
    let doc_raw = std::fs::read_to_string("schemas/document.schema.json").expect("read doc schema");
    let man_raw =
        std::fs::read_to_string("schemas/manifest.schema.json").expect("read manifest schema");
    let doc: serde_json::Value = serde_json::from_str(&doc_raw).expect("doc json");
    let man: serde_json::Value = serde_json::from_str(&man_raw).expect("man json");

    let doc_id = doc["$id"].as_str().expect("doc id");
    let man_id = man["$id"].as_str().expect("man id");
    assert_ne!(doc_id, man_id);

    let sample =
        std::fs::read_to_string("../../tests/fixtures/sample_project_v1.json").expect("fixture");
    craftcad_serialize::validate_document_json_str(&sample).expect("fixture valid");
}

#[test]
fn normalize_document_json_adds_defaults() {
    let mut v = serde_json::json!({
      "schema_version": 1,
      "id": "00000000-0000-4000-8000-000000000001",
      "units": "mm",
      "layers": [],
      "entities": [],
      "parts": [],
      "jobs": []
    });
    v = craftcad_serialize::normalize_document_json(v);
    assert!(v.get("materials").is_some());
    assert!(v.get("settings").is_some());
}
