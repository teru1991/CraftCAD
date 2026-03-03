use std::path::PathBuf;

#[path = "../../../src/testing/datasets_manifest.rs"]
mod datasets_manifest;

use datasets_manifest::{validate_manifest, Manifest};

#[test]
fn ssot_datasets_manifest_is_valid() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("ssot_lint crate must live under core/crates/ssot_lint")
        .to_path_buf();

    let manifest_path = repo_root.join("tests/datasets/manifest.json");
    let bytes = std::fs::read(&manifest_path).unwrap_or_else(|e| {
        panic!(
            "[DS_MANIFEST_READ] failed to read {}: {}",
            manifest_path.display(),
            e
        )
    });

    let manifest: Manifest = serde_json::from_slice(&bytes).unwrap_or_else(|e| {
        panic!(
            "[DS_MANIFEST_JSON_PARSE] failed to parse {}: {}",
            manifest_path.display(),
            e
        )
    });

    if let Err(error) = validate_manifest(&manifest, &repo_root) {
        panic!(
            "Datasets SSOT lint failed: {}\nmanifest_path={}\nrepo_root={}",
            error,
            manifest_path.display(),
            repo_root.display()
        );
    }
}
