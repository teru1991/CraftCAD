#[test]
fn migrate_v1_document_to_v2_adds_asset_fields() {
    let s = std::fs::read_to_string("tests/golden/diycad_document_v1.json").unwrap();
    let v = craftcad_serialize::validate_document_json_str(&s).unwrap();
    assert_eq!(v.get("schema_version").and_then(|x| x.as_u64()), Some(2));
    assert!(v.get("used_presets").unwrap().is_array());
    assert!(v.get("used_templates").unwrap().is_array());
    assert!(v.get("wizard_runs").unwrap().is_array());
}
