#[test]
fn migrate_v1_document_to_v2_adds_asset_fields() {
    let s = std::fs::read_to_string("../../tests/golden/diycad_document_v1.json").unwrap();
    let v = craftcad_serialize::validate_document_json_str(&s).unwrap();
    assert_eq!(v.get("schema_version").and_then(|x| x.as_u64()), Some(2));
    assert_eq!(v.get("used_presets").unwrap().as_array().unwrap().len(), 0);
    assert_eq!(
        v.get("used_templates").unwrap().as_array().unwrap().len(),
        0
    );
    assert_eq!(v.get("wizard_runs").unwrap().as_array().unwrap().len(), 0);
}

#[test]
fn normalize_dedups_used_asset_fields_deterministically() {
    let src = serde_json::json!({
      "schema_version": 2,
      "id": "00000000-0000-4000-8000-000000000001",
      "units": "mm",
      "layers": [],
      "entities": [],
      "parts": [],
      "jobs": [],
      "materials": [],
      "settings": {},
      "used_presets": [
        {"kind":"process","id":"cnc_basic","version":"1.0.0"},
        {"kind":"process","id":"cnc_basic","version":"1.0.0"}
      ],
      "used_templates": [
        {"id":"box_wizard","version":"1.0.0"},
        {"id":"box_wizard","version":"1.0.0"}
      ],
      "wizard_runs": [
        {"run_id":"a","template_id":"box_wizard","template_version":"1.0.0","inputs_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","seed":1,"outputs_part_ids":[],"created_at_unix_ms":1},
        {"run_id":"a","template_id":"box_wizard","template_version":"1.0.0","inputs_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","seed":1,"outputs_part_ids":[],"created_at_unix_ms":1}
      ]
    });
    let out = craftcad_serialize::normalize_document_json(src);
    assert_eq!(
        out.get("used_presets").unwrap().as_array().unwrap().len(),
        1
    );
    assert_eq!(
        out.get("used_templates").unwrap().as_array().unwrap().len(),
        1
    );
    assert_eq!(out.get("wizard_runs").unwrap().as_array().unwrap().len(), 1);
}
