use craftcad_io_support::{MappingRules, SupportMatrix};

#[test]
fn ssot_loads_cleanly() {
    let _sm = SupportMatrix::load_from_ssot().expect("SupportMatrix must load");
    let _mr = MappingRules::load_from_ssot().expect("MappingRules must load");
}
