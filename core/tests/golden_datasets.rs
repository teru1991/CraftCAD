use std::path::{Path, PathBuf};

#[path = "../src/testing/dataset_runner.rs"]
mod dataset_runner;
#[path = "../src/testing/datasets_manifest.rs"]
mod datasets_manifest;
#[path = "../src/testing/golden_harness.rs"]
mod golden_harness;

use dataset_runner::{run_dataset_by_id, NoiseMode};
use datasets_manifest::{CompareMode, ExpectedKind, Manifest};
use golden_harness::{
    compare_json_struct, compare_reason_codes, compare_svg_hash, DatasetMeta, InputRef,
};

fn repo_root() -> PathBuf {
    let core_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    core_dir.parent().expect("core/ parent").to_path_buf()
}

fn load_manifest(root: &Path) -> Manifest {
    let p = root.join("tests/datasets/manifest.json");
    let bytes = std::fs::read(&p).expect("read manifest.json");
    serde_json::from_slice(&bytes).expect("parse manifest.json")
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
            let expected_path = root.join(&exp.path);
            match (&exp.kind, &exp.compare) {
                (ExpectedKind::NormalizedModel, CompareMode::JsonStruct) => {
                    compare_json_struct(&root, &meta, &expected_path, out.normalized_model.clone())
                        .unwrap();
                }
                (ExpectedKind::Warnings, CompareMode::ReasonCodes) => {
                    compare_reason_codes(&root, &meta, &expected_path, out.warnings.clone()).unwrap();
                }
                (ExpectedKind::ExportedSvg, CompareMode::SvgHash) => {
                    compare_svg_hash(
                        &root,
                        &meta,
                        &expected_path,
                        &out.exported_svg.clone().expect("exported_svg not produced for dataset"),
                    )
                    .unwrap();
                }
                (ExpectedKind::ExportedJson, CompareMode::JsonStruct) => {
                    compare_json_struct(
                        &root,
                        &meta,
                        &expected_path,
                        out.exported_json.clone().expect("exported_json not produced for dataset"),
                    )
                    .unwrap();
                }
                (ExpectedKind::OpenResult, CompareMode::JsonStruct) => {
                    compare_json_struct(
                        &root,
                        &meta,
                        &expected_path,
                        out.extra_outputs
                            .get("open_result")
                            .cloned()
                            .expect("open_result not produced for dataset"),
                    )
                    .unwrap();
                }
                (ExpectedKind::SavedProject, CompareMode::JsonStruct) => {
                    compare_json_struct(
                        &root,
                        &meta,
                        &expected_path,
                        out.extra_outputs
                            .get("saved_project")
                            .cloned()
                            .expect("saved_project not produced for dataset"),
                    )
                    .unwrap();
                }
                (ExpectedKind::NestResult, CompareMode::JsonStruct) => {
                    compare_json_struct(
                        &root,
                        &meta,
                        &expected_path,
                        out.extra_outputs
                            .get("nest_result")
                            .cloned()
                            .expect("nest_result not produced for dataset"),
                    )
                    .unwrap();
                }
                other => panic!(
                    "Unsupported expected combination in Step3/5 (binary-free): {other:?}"
                ),
            }
        }
    }
}
