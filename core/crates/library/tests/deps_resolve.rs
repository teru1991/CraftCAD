use craftcad_library::deps::{resolve_required_presets, WizardTemplateMinimal};
use craftcad_presets::PresetsService;
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
fn template_required_presets_resolve_ok() {
    let repo = repo_root_from_manifest();
    let user_tmp = tempfile::tempdir().unwrap();
    let svc = PresetsService::new(repo.clone(), user_tmp.path().to_path_buf()).unwrap();

    let s = std::fs::read_to_string(
        repo.join("docs")
            .join("specs")
            .join("templates")
            .join("shelf_wizard.template.json"),
    )
    .unwrap();
    let tpl: WizardTemplateMinimal = serde_json::from_str(&s).unwrap();

    resolve_required_presets(&svc, &tpl).unwrap();
}
