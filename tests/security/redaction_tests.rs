use craftcad_security::redact_str;

#[test]
fn masks_email() {
    let s = redact_str("mail me a@b.com");
    assert!(s.contains("<EMAIL>"));
}
