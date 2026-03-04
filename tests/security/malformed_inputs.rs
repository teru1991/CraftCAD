#[test]
fn redact_json_handles_malformed_like_input() {
    let red = security::Redactor::from_ssot(security::RedactorConfig {
        limits_profile: security::LimitsProfile::Default,
    })
    .expect("redactor");
    let out = red.redact_json(&serde_json::json!({"token":"secret","nested":["a",{"k":"v"}]}));
    assert!(out.is_object());
}
