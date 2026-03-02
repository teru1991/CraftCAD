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
    panic!("repo root not found");
}

fn normalize_seed(mut v: serde_json::Value) -> serde_json::Value {
    if let Some(nj) = v.get_mut("recommended_nest_job") {
        if let Some(obj) = nj.as_object_mut() {
            obj.insert("seed".to_string(), serde_json::Value::Number(0.into()));
        }
    }
    v
}

fn hash_json(v: &serde_json::Value) -> String {
    let s = serde_json::to_string(v).unwrap();
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

#[test]
fn shelf_wizard_is_deterministic_10_runs() {
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

    let mut hashes = vec![];
    for _ in 0..10 {
        let parts = eng
            .run_wizard_parts("shelf_wizard.template.json", &wi)
            .unwrap();
        let v = normalize_seed(serde_json::to_value(parts).unwrap());
        hashes.push(hash_json(&v));
    }
    for h in &hashes[1..] {
        assert_eq!(&hashes[0], h, "hash mismatch => non-deterministic output");
    }
}
