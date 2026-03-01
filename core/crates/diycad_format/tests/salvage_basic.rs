use craftcad_diycad_format::salvage_document;
use serde_json::json;

#[test]
fn test_salvage_document_valid_passes_through() {
    // Document with all required fields should pass through
    let input = json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "schema_version": 1,
        "created_by": "test_user",
        "updated_at": "2026-03-01T00:00:00Z",
        "title": "Test Document",
        "parts": [],
        "nest_jobs": [],
        "settings": {}
    });

    let result = salvage_document(&input);
    assert!(result.is_ok(), "Salvage should succeed for valid document");

    let (_salvaged_doc, report) = result.unwrap();
    assert!(report.recovered, "Report should indicate recovery");
    assert_eq!(
        report.normalized_fields.len(),
        0,
        "No fields should be normalized for valid document"
    );
}

#[test]
fn test_salvage_document_missing_required_field() {
    // Document missing required 'id' field should fail
    let input = json!({
        "schema_version": 1,
        "created_by": "test_user",
        "updated_at": "2026-03-01T00:00:00Z",
        "title": "Test Document",
        "parts": [],
        "nest_jobs": [],
        "settings": {}
    });

    let result = salvage_document(&input);
    assert!(
        result.is_err(),
        "Salvage should fail for missing required 'id' field"
    );

    let err = result.unwrap_err();
    assert_eq!(err.code, "SALVAGE_DOCUMENT_MALFORMED");
}

#[test]
fn test_salvage_document_invalid_structure() {
    // Non-object JSON should fail
    let input = json!([1, 2, 3]);

    let result = salvage_document(&input);
    assert!(result.is_err(), "Salvage should fail for non-object JSON");

    let err = result.unwrap_err();
    assert_eq!(err.code, "SALVAGE_DOCUMENT_MALFORMED");
}

#[test]
fn test_salvage_document_with_extra_properties() {
    // Document with extra properties (not in schema) should fail validation
    // (schema has additionalProperties: false)
    let input = json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "schema_version": 1,
        "created_by": "test_user",
        "updated_at": "2026-03-01T00:00:00Z",
        "title": "Test Document",
        "parts": [],
        "nest_jobs": [],
        "settings": {},
        "extra_field": "should_fail"
    });

    let result = salvage_document(&input);
    // This should fail because of additionalProperties: false in schema
    assert!(
        result.is_err(),
        "Salvage should fail for document with extra properties"
    );
}

