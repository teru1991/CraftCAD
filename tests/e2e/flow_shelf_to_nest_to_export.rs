use craftcad_wizards::parts::model::PartsDraft;
use craftcad_wizards::types::WizardInput;
use craftcad_wizards::WizardEngine;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let start = std::env::current_dir().unwrap();
    for up in 0..=6usize {
        let mut p = start.clone();
        for _ in 0..up {
            p = p.parent().unwrap_or(&p).to_path_buf();
        }
        if p.join("docs").join("specs").exists() {
            return p;
        }
    }
    panic!("repo root not found from {:?}", start);
}

fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

fn normalize_seed(mut v: serde_json::Value) -> serde_json::Value {
    if let Some(nj) = v.get_mut("recommended_nest_job") {
        if let Some(obj) = nj.as_object_mut() {
            obj.insert("seed".to_string(), serde_json::Value::Number(0.into()));
        }
    }
    v
}

#[test]
fn flow_shelf_to_nest_to_export_internal_json() {
    let repo = repo_root();
    let user_tmp = tempfile::tempdir().unwrap();
    let eng = WizardEngine::new(repo.clone(), user_tmp.path().to_path_buf()).unwrap();

    let mut inputs = BTreeMap::new();
    inputs.insert("width_mm".into(), serde_json::json!(600.0));
    inputs.insert("depth_mm".into(), serde_json::json!(300.0));
    inputs.insert("thickness_mm".into(), serde_json::json!(18.0));
    inputs.insert("quantity".into(), serde_json::json!(1));
    inputs.insert("hole_dowel".into(), serde_json::json!(true));
    inputs.insert("dowel_diameter_mm".into(), serde_json::json!(8.0));
    inputs.insert("offset_mm".into(), serde_json::json!(35.0));

    let wi = WizardInput {
        template_id: "shelf_wizard".into(),
        inputs,
        seed: None,
    };

    let parts: PartsDraft = eng
        .run_wizard_parts("shelf_wizard.template.json", &wi)
        .unwrap();

    assert!(
        parts.recommended_nest_job.is_some(),
        "recommended_nest_job must exist"
    );

    let out_dir = user_tmp.path().join("exports");
    std::fs::create_dir_all(&out_dir).unwrap();
    let out_path = out_dir.join("shelf_export.json");

    let json = normalize_seed(serde_json::to_value(&parts).unwrap());
    std::fs::write(&out_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
    assert!(out_path.exists());

    let s = std::fs::read_to_string(&out_path).unwrap();
    let h = sha256_hex(&s);

    let expected = "d8567e43d318f4200475f1a7ac60e90f53fd6e31d49be9f36601f8e5293c5f7c";
    assert_eq!(
        h, expected,
        "export hash changed; this may break determinism/compat; got={h}"
    );
}
