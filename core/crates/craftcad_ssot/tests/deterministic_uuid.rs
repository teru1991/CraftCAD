use craftcad_ssot::deterministic_uuid;

#[test]
fn deterministic_uuid_is_stable_for_same_inputs() {
    let a = deterministic_uuid("material", "demo");
    let b = deterministic_uuid("material", "demo");
    assert_eq!(a, b);
}

#[test]
fn deterministic_uuid_changes_with_tag_or_key() {
    let base = deterministic_uuid("material", "demo");
    assert_ne!(base, deterministic_uuid("part", "demo"));
    assert_ne!(base, deterministic_uuid("material", "demo2"));
}
