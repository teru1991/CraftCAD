use craftcad_io_support::MappingRules;

#[test]
fn test_mapping_rules_load_and_behavior() {
    let mr = MappingRules::load_from_ssot().expect("load mapping rules");

    // alias hit
    assert_eq!(mr.map_layer(" cut "), "CUT");
    assert_eq!(mr.map_linetype("solid"), "CONTINUOUS");

    // unknown layer is preserved (normalized)
    assert_eq!(mr.map_layer("my layer"), "MY_LAYER");

    // unknown linetype falls back
    assert_eq!(mr.map_linetype("my_linetype"), "CONTINUOUS");
}
