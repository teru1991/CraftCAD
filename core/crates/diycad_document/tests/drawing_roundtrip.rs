use diycad_document::{load_document_json, save_document_json};
use drawing_model::DrawingDoc;

#[test]
fn roundtrip_preserves_unknown_fields_and_keeps_drawing() {
    let original = serde_json::json!({
      "schema_version": 10,
      "project": { "name": "demo" },
      "parts": [{ "id": "PART_000001" }],
      "nest": { "id": "NEST_000001" }
    });

    let mut drawing = DrawingDoc::new_minimal("DRW_000001");
    drawing.refs.parts = vec!["PART_000001".to_string()];
    drawing.refs.sketches = vec!["SK_000001".to_string()];
    drawing.dimensions = vec![];
    drawing.annotations = vec![];

    let saved = save_document_json(original.clone(), Some(&drawing)).expect("save failed");

    let loaded = load_document_json(&saved).expect("load failed");
    assert!(
        loaded.warnings.is_empty(),
        "warnings must be empty but got: {:?}",
        loaded.warnings
    );

    let doc = &loaded.document;
    assert_eq!(
        doc.pointer("/project/name").and_then(|v| v.as_str()),
        Some("demo")
    );
    assert!(doc.get("parts").is_some());
    assert!(doc.get("nest").is_some());

    assert!(loaded.drawing.is_some());
    assert_eq!(
        loaded.drawing.as_ref().map(|d| d.id.as_str()),
        Some("DRW_000001")
    );
}
