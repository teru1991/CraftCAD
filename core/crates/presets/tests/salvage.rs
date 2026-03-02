use craftcad_presets::model::PresetKind;
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
fn salvage_returns_matching_builtin_versions() {
    let repo = repo_root_from_manifest();
    let tmp = tempfile::tempdir().unwrap();
    let svc = PresetsService::new(repo, tmp.path().to_path_buf()).unwrap();

    let s = svc
        .salvage_missing_preset(PresetKind::Material, "plywood_18mm", "^1.0")
        .unwrap();
    assert_eq!(s.missing_kind, "material");
    assert_eq!(s.fallback_builtin_versions, vec!["1.0.0"]);
}

#[test]
fn salvage_empty_when_id_missing() {
    let repo = repo_root_from_manifest();
    let tmp = tempfile::tempdir().unwrap();
    let svc = PresetsService::new(repo, tmp.path().to_path_buf()).unwrap();

    let s = svc
        .salvage_missing_preset(PresetKind::Output, "missing_id", "^1.0")
        .unwrap();
    assert!(s.fallback_builtin_versions.is_empty());
}
