use std::path::{Path, PathBuf};

#[path = "../../../src/testing/dataset_runner.rs"]
mod dataset_runner;

#[path = "../../../src/testing/datasets_manifest.rs"]
mod datasets_manifest;
#[path = "../../../src/testing/golden_harness.rs"]
mod golden_harness;

use dataset_runner::{run_dataset_by_id, NoiseMode};

use datasets_manifest::{CompareMode, ExpectedKind, Manifest};
use golden_harness::{
    compare_json_struct, compare_reason_codes, compare_svg_hash, DatasetMeta, InputRef,
};

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

fn build_meta(ds: &datasets_manifest::Dataset) -> DatasetMeta {
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
fn golden_datasets_smoke() {
    let root = repo_root();
    let manifest = load_manifest(&root);
    for ds in &manifest.datasets {
        if !ds.tags.iter().any(|t| t == "smoke") {
            continue;
        }
        let meta = build_meta(ds);
        let out = run_dataset_by_id(&root, &manifest, &ds.id, NoiseMode::Off).unwrap();
        for exp in &ds.expected {
            let ep = root.join(&exp.path);
            match (&exp.kind, &exp.compare) {
                (ExpectedKind::NormalizedModel, CompareMode::JsonStruct) => {
                    compare_json_struct(&root, &meta, &ep, out.normalized_model.clone()).unwrap()
                }
                (ExpectedKind::Warnings, CompareMode::ReasonCodes) => {
                    compare_reason_codes(&root, &meta, &ep, out.warnings.clone()).unwrap()
                }
                (ExpectedKind::ExportedSvg, CompareMode::SvgHash) => {
                    compare_svg_hash(&root, &meta, &ep, &out.exported_svg.clone().unwrap()).unwrap()
                }
                (ExpectedKind::ExportedJson, CompareMode::JsonStruct) => {
                    compare_json_struct(&root, &meta, &ep, out.exported_json.clone().unwrap())
                        .unwrap()
                }
                (ExpectedKind::OpenResult, CompareMode::JsonStruct) => compare_json_struct(
                    &root,
                    &meta,
                    &ep,
                    out.extra_outputs.get("open_result").cloned().unwrap(),
                )
                .unwrap(),
                (ExpectedKind::SavedProject, CompareMode::JsonStruct) => compare_json_struct(
                    &root,
                    &meta,
                    &ep,
                    out.extra_outputs.get("saved_project").cloned().unwrap(),
                )
                .unwrap(),
                (ExpectedKind::NestResult, CompareMode::JsonStruct) => compare_json_struct(
                    &root,
                    &meta,
                    &ep,
                    out.extra_outputs.get("nest_result").cloned().unwrap(),
                )
                .unwrap(),
                _ => panic!("unsupported kind/comparison"),
            }
        }
    }
}
