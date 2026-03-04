use security::{LimitsProfile, Redactor, RedactorConfig};
use serde_json::json;

#[test]
fn redaction_golden_examples() {
    let red = Redactor::from_ssot(RedactorConfig {
        limits_profile: LimitsProfile::Default,
    })
    .unwrap();
    let input = json!({
        "email": "user@example.com",
        "path": "C:\\Users\\alice\\secret\\a.diycad",
        "note": "hello this is private memo. token=ABCDEF123456",
        "x": "Bearer SUPERSECRET",
        "url": "https://example.com/?token=XYZ"
    });
    let out = red.redact_json(&input);
    let s = out.to_string();
    assert!(!s.contains("user@example.com"));
    assert!(!s.contains("SUPERSECRET"));
    assert!(!s.contains("C:\\Users\\alice"));
    assert!(s.contains("<REDACTED>") || s.contains("<EMAIL>") || s.contains("<URL>"));
    assert!(!s.contains("hello this is private memo"));
    assert!(s.contains("free_text:hash=sha256:"));
}
