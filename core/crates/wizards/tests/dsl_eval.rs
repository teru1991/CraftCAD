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
fn eval_accepts_safe_expressions_in_step5() {
    let repo = repo_root_from_manifest();
    let user_tmp = tempfile::tempdir().unwrap();
    let eng = WizardEngine::new(repo.clone(), user_tmp.path().to_path_buf()).unwrap();

    let mut inputs = BTreeMap::new();
    inputs.insert("inner_w_mm".into(), json!(200.0));
    inputs.insert("inner_h_mm".into(), json!(120.0));
    inputs.insert("inner_d_mm".into(), json!(120.0));
    inputs.insert("thickness_mm".into(), json!(12.0));
    inputs.insert("lid".into(), json!(false));

    let wi = WizardInput {
        template_id: "box_wizard".into(),
        inputs,
        seed: None,
    };
    let r = eng
        .run_template_draft("box_wizard.template.json", &wi)
        .unwrap();
    let rect = r
        .evaluated_ops
        .iter()
        .find(|o| o.op == "add_part_rect")
        .expect("rect op exists");
    assert_eq!(rect.args.get("w_mm").and_then(|v| v.as_f64()), Some(224.0));
    assert_eq!(rect.args.get("h_mm").and_then(|v| v.as_f64()), Some(144.0));
}
