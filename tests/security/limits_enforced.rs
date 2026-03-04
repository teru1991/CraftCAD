#[test]
fn loads_limits() {
    let limits = security::Limits::load_from_ssot(security::LimitsProfile::Default)
        .expect("limits should load");
    assert!(limits.max_import_bytes > 0);
}
