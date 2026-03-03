use std::path::PathBuf;

#[path = "../src/testing/datasets_manifest.rs"]
mod datasets_manifest;

use datasets_manifest::{validate_manifest, Manifest};

#[test]
fn ssot_datasets_manifest_is_valid() {
    let core_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = core_dir.parent().expect("core/ must have parent").to_path_buf();

    let manifest_path = repo_root.join("tests/datasets/manifest.json");
    let bytes = std::fs::read(&manifest_path).expect("failed to read tests/datasets/manifest.json");

    let manifest: Manifest = serde_json::from_slice(&bytes).expect("manifest.json must be valid JSON");

    if let Err(error) = validate_manifest(&manifest, &repo_root) {
        panic!(
            "Datasets SSOT lint failed: {}\nmanifest_path={}\nrepo_root={}",
            error,
            manifest_path.display(),
            repo_root.display()
        );
    }
}
