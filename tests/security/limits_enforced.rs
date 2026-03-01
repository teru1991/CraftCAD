use craftcad_security::load_limits;

#[test]
fn loads_limits() {
    let limits = load_limits("docs/specs/security/limits.json").unwrap();
    assert!(limits.max_import_bytes > 0);
}
