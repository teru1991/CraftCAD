use std::path::{Path, PathBuf};

use core::testing::dataset_runner::{run_dataset_by_id, NoiseMode};
use core::testing::datasets_manifest::Manifest;
use core::testing::determinism_harness::assert_deterministic;
use core::testing::golden_harness::{DatasetMeta, InputRef};

fn repo_root() -> PathBuf {
    let core_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    core_dir.parent().unwrap().to_path_buf()
}

fn load_manifest(root: &Path) -> Manifest {
    let p = root.join("tests/datasets/manifest.json");
    let bytes = std::fs::read(&p).expect("read manifest.json");
    serde_json::from_slice(&bytes).expect("parse manifest.json")
}

fn to_meta(ds: &core::testing::datasets_manifest::Dataset) -> DatasetMeta {
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
fn determinism_migrate_10x_project_json() {
    let root = repo_root();
    std::env::set_var("CRAFTCAD_FAILURE_ARTIFACTS_DIR", root.join("failure_artifacts"));

    let manifest = load_manifest(&root);

    for ds in &manifest.datasets {
        if !ds.tags.iter().any(|t| t == "determinism") {
            continue;
        }
        if !ds.id.contains("project_json") {
            continue;
        }

        let meta = to_meta(ds);
        let dsid = ds.id.clone();

        let mut runner = || {
            let out = run_dataset_by_id(&root, &manifest, &dsid, NoiseMode::Off)
                .expect("run_dataset_by_id failed");
            (
                out.normalized_model,
                out.warnings,
                out.exported_svg,
                out.exported_json,
            )
        };
        assert_deterministic(&root, &meta, 10, &mut runner).unwrap();

        let noise_seed = meta.seed.wrapping_add(999);
        let mut runner = || {
            let out = run_dataset_by_id(&root, &manifest, &dsid, NoiseMode::On(noise_seed))
                .expect("run_dataset_by_id failed");
            (
                out.normalized_model,
                out.warnings,
                out.exported_svg,
                out.exported_json,
            )
        };
        assert_deterministic(&root, &meta, 10, &mut runner).unwrap();
    }
}
