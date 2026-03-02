use craftcad_wizards::engine::validate::validate_inputs;
use craftcad_wizards::template::TemplateRegistry;
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn input_rejects_unknown_key() {
    let reg = TemplateRegistry::new(None).unwrap();
    let t = reg
        .load_builtin_template("shelf_wizard.template.json")
        .unwrap();

    let mut inputs = BTreeMap::new();
    inputs.insert("width_mm".to_string(), json!(600.0));
    inputs.insert("unknown".to_string(), json!(1));

    let e = validate_inputs(&t, &inputs).unwrap_err();
    assert!(format!("{:?}", e).contains("unknown input key"));
}
