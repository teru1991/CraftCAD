use jsonschema::JSONSchema;
use serde_json::Value;

fn compile_schema(schema_json: &Value) -> JSONSchema {
    JSONSchema::options()
        .compile(schema_json)
        .expect("schema compile failed")
}

#[test]
fn drawing_doc_schema_accepts_sample() {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../../docs/specs/project_file/drawing_doc.schema.json"
    ))
    .expect("failed to read schema file");

    let instance: Value = serde_json::json!({
      "schema_version": 1,
      "id": "DRW_000001",
      "units": "mm",
      "view": { "model_to_sheet": { "scale": 1.0, "translate_mm": { "x": 0.0, "y": 0.0 } } },
      "style_preset_id": "default_v1",
      "sheet_template_id": "a4_portrait_v1",
      "print_preset_id": "a4_default_v1",
      "dimensions": [],
      "annotations": [],
      "refs": { "parts": ["PART_000001"], "sketches": ["SK_000001"] }
    });

    let compiled = compile_schema(&schema);
    let validation = compiled.validate(&instance);
    if let Err(errors) = validation {
        let msgs: Vec<String> = errors
            .map(|e| format!("{} @ {}", e, e.instance_path))
            .collect();
        panic!("schema validation failed:\n{}", msgs.join("\n"));
    }
}
