use craftcad_wizards::template::TemplateRegistry;

#[test]
fn load_templates_schema_ok() {
    let reg = TemplateRegistry::new(None).unwrap();
    let t = reg
        .load_builtin_template("shelf_wizard.template.json")
        .unwrap();
    assert_eq!(t.template_id, "shelf_wizard");
}
