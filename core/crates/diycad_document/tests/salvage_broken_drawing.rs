use diycad_document::load_document_json;

#[test]
fn broken_drawing_is_salvaged_and_project_still_loads() {
    let broken = r#"
    {
      "schema_version": 10,
      "project": { "name": "demo" },
      "parts": [{ "id": "PART_000001" }],
      "drawing": "THIS_IS_BROKEN"
    }
    "#;

    let loaded = load_document_json(broken).expect("load failed");
    assert!(loaded.drawing.is_none(), "drawing must be salvaged to None");
    assert!(loaded.document.get("parts").is_some());

    let codes: Vec<&'static str> = loaded.warnings.iter().map(|w| w.reason_code()).collect();
    let mut sorted = codes.clone();
    sorted.sort();
    assert_eq!(
        codes, sorted,
        "warnings must be returned in stable sorted order"
    );
    assert!(codes.contains(&"CAD_DRAWING_SALVAGED_READONLY"));
    assert!(codes.contains(&"CAD_DRAWING_PARSE_FAILED"));
}

#[test]
fn unsupported_drawing_schema_is_salvaged_with_stable_warning_order() {
    let broken = r#"
    {
      "schema_version": 10,
      "project": { "name": "demo" },
      "parts": [{ "id": "PART_000001" }],
      "drawing": {
        "schema_version": 99,
        "id": "DRW_000001",
        "units": "mm",
        "view": { "model_to_sheet": { "scale": 1.0, "translate_mm": { "x": 0.0, "y": 0.0 } } },
        "style_preset_id": "default_v1",
        "sheet_template_id": "a4_portrait_v1",
        "print_preset_id": "a4_default_v1",
        "dimensions": [],
        "annotations": [],
        "refs": { "parts": ["PART_000001"], "sketches": ["SK_000001"] }
      }
    }
    "#;

    let loaded = load_document_json(broken).expect("load failed");
    assert!(loaded.drawing.is_none());
    let codes: Vec<&'static str> = loaded.warnings.iter().map(|w| w.reason_code()).collect();
    assert_eq!(
        codes,
        vec![
            "CAD_DRAWING_SALVAGED_READONLY",
            "CAD_DRAWING_UNSUPPORTED_SCHEMA"
        ]
    );
}
