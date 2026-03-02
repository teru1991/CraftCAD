use craftcad_wizards::types::WizardInput;
use craftcad_wizards::WizardEngine;
use serde_json::Value;
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

fn read_json(p: &PathBuf) -> Value {
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn normalize_for_golden(mut v: Value) -> Value {
    if let Some(nj) = v.get_mut("recommended_nest_job") {
        if let Some(obj) = nj.as_object_mut() {
            obj.insert("seed".to_string(), Value::Number(0.into()));
        }
    }
    v
}

#[test]
fn box_golden() {
    let repo = repo_root_from_manifest();
    let user_tmp = tempfile::tempdir().unwrap();
    let eng = WizardEngine::new(repo.clone(), user_tmp.path().to_path_buf()).unwrap();

    let input_path = repo
        .join("tests")
        .join("golden")
        .join("wizards")
        .join("box_input_01.json");
    let expected_path = repo
        .join("tests")
        .join("golden")
        .join("wizards")
        .join("box_expected_01.json");

    let wi: WizardInput = serde_json::from_value(read_json(&input_path)).unwrap();
    let parts = eng
        .run_wizard_parts("box_wizard.template.json", &wi)
        .unwrap();
    let got = normalize_for_golden(serde_json::to_value(parts).unwrap());
    let exp = normalize_for_golden(read_json(&expected_path));
    assert_eq!(got, exp);
}
