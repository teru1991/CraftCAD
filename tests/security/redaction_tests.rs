#[test]
fn redactor_masks_email() {
    let red = security::Redactor::from_ssot(security::RedactorConfig {
        limits_profile: security::LimitsProfile::Default,
    })
    .expect("redactor");
    let out = red.redact_str("contact me at user@example.com");
    assert!(out.contains("<EMAIL>") || out.contains("free_text"));
}
