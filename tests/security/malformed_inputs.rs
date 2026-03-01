use craftcad_security::redact_json;

#[test]
fn malformed_input_no_panic() {
    let _ = redact_json(serde_json::json!({"x": ["a", 1, {"y": "b"}]}));
}
