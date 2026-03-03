use std::path::{Path, PathBuf};

#[path = "../../../src/testing/dataset_runner.rs"]
mod dataset_runner;
#[path = "../../../src/testing/datasets_manifest.rs"]
mod datasets_manifest;
#[path = "../../../src/testing/determinism_harness.rs"]
mod determinism_harness;
#[path = "../../../src/testing/golden_harness.rs"]
mod golden_harness;

use dataset_runner::{run_dataset_by_id, NoiseMode};
use datasets_manifest::Manifest;
use determinism_harness::assert_deterministic;
use golden_harness::{DatasetMeta, InputRef};

fn repo_root() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf()
}
fn load_manifest(root: &Path) -> Manifest {
    serde_json::from_slice(&std::fs::read(root.join("tests/datasets/manifest.json")).unwrap())
        .unwrap()
}
fn to_meta(ds: &datasets_manifest::Dataset) -> DatasetMeta {
    DatasetMeta {
        id: ds.id.clone(),
        seed: ds.seed,
        epsilon: ds.determinism.epsilon,
        round_step: ds.determinism.round_step,
        ordering_tag: ds.determinism.ordering_tag.clone(),
        limits_ref: ds.limits_ref.clone(),
        inputs: ds
            .inputs
            .iter()
            .map(|i| InputRef {
                kind: format!("{:?}", i.kind),
                path: i.path.clone(),
                sha256: i.sha256.clone(),
            })
            .collect(),
    }
}

#[test]
fn determinism_wizard_10x() {
    let root = repo_root();
    std::env::set_var(
        "CRAFTCAD_FAILURE_ARTIFACTS_DIR",
        root.join("failure_artifacts"),
    );
    let manifest = load_manifest(&root);
    for ds in &manifest.datasets {
        if !ds.tags.iter().any(|t| t == "determinism") || !ds.id.starts_with("wizard_") {
            continue;
        }
        let meta = to_meta(ds);
        let dsid = ds.id.clone();
        let mut r = || {
            let out = run_dataset_by_id(&root, &manifest, &dsid, NoiseMode::Off).unwrap();
            (
                out.normalized_model,
                out.warnings,
                out.exported_svg,
                out.exported_json,
            )
        };
        assert_deterministic(&root, &meta, 10, &mut r).unwrap();
        let mut r = || {
            let out = run_dataset_by_id(
                &root,
                &manifest,
                &dsid,
                NoiseMode::On(meta.seed.wrapping_add(999)),
            )
            .unwrap();
            (
                out.normalized_model,
                out.warnings,
                out.exported_svg,
                out.exported_json,
            )
        };
        assert_deterministic(&root, &meta, 10, &mut r).unwrap();
    }
}
