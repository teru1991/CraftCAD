use craftcad_wizards::types::WizardInput;
use craftcad_wizards::WizardEngine;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn repo_root_from_manifest() -> PathBuf {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for up in 0..=10usize {
        let mut p = start.clone();
        for _ in 0..up {
            p = p.parent().unwrap_or(&p).to_path_buf();
        }
        if p.join("docs").join("specs").exists() {
            return p;
        }
    }
    panic!("repo root not found");
}

#[test]
fn derived_seed_is_stable() {
    let repo = repo_root_from_manifest();
    let user_tmp = tempfile::tempdir().unwrap();
    let eng = WizardEngine::new(repo.clone(), user_tmp.path().to_path_buf()).unwrap();

    let mut inputs = BTreeMap::new();
    inputs.insert("width_mm".into(), json!(600.0));
    inputs.insert("depth_mm".into(), json!(300.0));
    inputs.insert("thickness_mm".into(), json!(18.0));
    inputs.insert("quantity".into(), json!(1));
    inputs.insert("hole_dowel".into(), json!(true));
    inputs.insert("dowel_diameter_mm".into(), json!(8.0));
    inputs.insert("offset_mm".into(), json!(35.0));

    let wi = WizardInput {
        template_id: "shelf_wizard".into(),
        inputs,
        seed: None,
    };
    let r1 = eng
        .run_template_draft("shelf_wizard.template.json", &wi)
        .unwrap();
    let r2 = eng
        .run_template_draft("shelf_wizard.template.json", &wi)
        .unwrap();
    assert_eq!(r1.seed_used, r2.seed_used);
    assert_eq!(
        serde_json::to_string(&r1.evaluated_ops).unwrap(),
        serde_json::to_string(&r2.evaluated_ops).unwrap()
    );
}
