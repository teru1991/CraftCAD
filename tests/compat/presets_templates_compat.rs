use craftcad_presets::PresetsService;
use craftcad_wizards::template::TemplateRegistry;
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

#[test]
fn builtin_presets_and_templates_load() {
    let repo = repo_root();
    let user_tmp = tempfile::tempdir().unwrap();

    let _svc = PresetsService::new(repo.clone(), user_tmp.path().to_path_buf()).unwrap();

    let reg = TemplateRegistry::new(Some(repo.clone())).unwrap();
    let _ = reg
        .load_builtin_template("shelf_wizard.template.json")
        .unwrap();
    let _ = reg
        .load_builtin_template("box_wizard.template.json")
        .unwrap();
    let _ = reg
        .load_builtin_template("leather_pouch_wizard.template.json")
        .unwrap();
}
