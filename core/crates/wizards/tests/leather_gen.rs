use craftcad_wizards::parts::model::{Feature2D, Outline2D};
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
    if let Some(parts) = v.get_mut("parts").and_then(|x| x.as_array_mut()) {
        for p in parts {
            if let Some(obj) = p.as_object_mut() {
                obj.insert("features".to_string(), Value::Array(vec![]));
            }
        }
    }
    v
}

#[test]
fn leather_golden_and_stats() {
    let repo = repo_root_from_manifest();
    let user_tmp = tempfile::tempdir().unwrap();
    let eng = WizardEngine::new(repo.clone(), user_tmp.path().to_path_buf()).unwrap();

    let input_path = repo
        .join("tests")
        .join("golden")
        .join("wizards")
        .join("leather_input_01.json");
    let expected_path = repo
        .join("tests")
        .join("golden")
        .join("wizards")
        .join("leather_expected_01.json");

    let wi: WizardInput = serde_json::from_value(read_json(&input_path)).unwrap();
    let parts = eng
        .run_wizard_parts("leather_pouch_wizard.template.json", &wi)
        .unwrap();

    let p0 = &parts.parts[0];
    assert!(p0.features.len() > 10);
    let mut minx = f64::INFINITY;
    let mut miny = f64::INFINITY;
    let mut maxx = f64::NEG_INFINITY;
    let mut maxy = f64::NEG_INFINITY;
    for f in &p0.features {
        if let Feature2D::StitchHole { x_mm, y_mm, .. } = *f {
            minx = minx.min(x_mm);
            miny = miny.min(y_mm);
            maxx = maxx.max(x_mm);
            maxy = maxy.max(y_mm);
        }
    }

    let (w, h) = match p0.outline {
        Outline2D::Rect { w_mm, h_mm } => (w_mm, h_mm),
    };
    assert!(minx >= 0.0 && miny >= 0.0);
    assert!(maxx <= w && maxy <= h);

    let got = normalize_for_golden(serde_json::to_value(parts).unwrap());
    let exp = normalize_for_golden(read_json(&expected_path));
    assert_eq!(got, exp);
}
