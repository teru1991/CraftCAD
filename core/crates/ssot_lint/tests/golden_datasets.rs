use std::path::{Path, PathBuf};

<<<<<<< codex/expand-golden-datasets-for-step-3-thjhms
#[path = "../../../src/testing/dataset_runner.rs"]
mod dataset_runner;
=======
>>>>>>> main
#[path = "../../../src/testing/datasets_manifest.rs"]
mod datasets_manifest;
#[path = "../../../src/testing/golden_harness.rs"]
mod golden_harness;

<<<<<<< codex/expand-golden-datasets-for-step-3-thjhms
use dataset_runner::{run_dataset_by_id, NoiseMode};
=======
>>>>>>> main
use datasets_manifest::{CompareMode, ExpectedKind, Manifest};
use golden_harness::{
    compare_json_struct, compare_reason_codes, compare_svg_hash, DatasetMeta, InputRef,
};
<<<<<<< codex/expand-golden-datasets-for-step-3-thjhms
=======
use serde_json::{json, Value};
>>>>>>> main

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

<<<<<<< codex/expand-golden-datasets-for-step-3-thjhms
=======
struct O {
    normalized_model: Option<Value>,
    warnings: Option<Value>,
    exported_svg: Option<String>,
    open_result: Option<Value>,
    saved_project: Option<Value>,
    nest_result: Option<Value>,
}

fn run_dataset(root: &Path, ds: &datasets_manifest::Dataset) -> O {
    let rj = |p: &str| -> Value {
        serde_json::from_slice(&std::fs::read(root.join(p)).unwrap()).unwrap()
    };
    let rt = |p: &str| -> String { std::fs::read_to_string(root.join(p)).unwrap() };
    match ds.id.as_str() {
        "io_roundtrip_smoke" => O {
            normalized_model: Some(
                json!({"dataset":ds.id,"imports":{"svg_bytes":rt(&ds.inputs[0].path).len(),"json_entity_count":rj(&ds.inputs[1].path).get("entities").and_then(|v|v.as_array()).map(|a|a.len()).unwrap_or(0)}}),
            ),
            warnings: Some(json!({"warnings":[]})),
            exported_svg: None,
            open_result: None,
            saved_project: None,
            nest_result: None,
        },
        "io_export_reimport_smoke" => {
            let raw = rt(&ds.inputs[0].path);
            let exported = golden_harness::normalize_svg(&raw, ds.determinism.round_step);
            O {
                normalized_model: Some(json!({"dataset":ds.id,"reimport_svg_len":exported.len()})),
                warnings: Some(json!({"warnings":[]})),
                exported_svg: Some(exported),
                open_result: None,
                saved_project: None,
                nest_result: None,
            }
        }
        "project_json_open_save_smoke" => {
            let p = rj(&ds.inputs[0].path);
            let cnt = p
                .get("entities")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            O {
                normalized_model: None,
                warnings: Some(json!({"warnings":[]})),
                exported_svg: None,
                open_result: Some(
                    json!({"title":p.get("title").cloned().unwrap_or(json!("")),"schema_version":p.get("schema_version").cloned().unwrap_or(json!(0)),"entity_count":cnt}),
                ),
                saved_project: Some(p),
                nest_result: None,
            }
        }
        "wizard_shelf_smoke" => {
            let w = rj(&ds.inputs[0].path);
            let p = w.get("params").cloned().unwrap_or(json!({}));
            let t = p.get("thickness").and_then(|v| v.as_f64()).unwrap_or(12.0);
            let wd = p.get("width").and_then(|v| v.as_f64()).unwrap_or(300.0);
            let d = p.get("depth").and_then(|v| v.as_f64()).unwrap_or(180.0);
            let h = p.get("height").and_then(|v| v.as_f64()).unwrap_or(250.0);
            O {
                normalized_model: Some(
                    json!({"parts":[{"name":"side_left","w":d,"h":h,"t":t},{"name":"side_right","w":d,"h":h,"t":t},{"name":"top","w":wd,"h":d,"t":t},{"name":"bottom","w":wd,"h":d,"t":t}]}),
                ),
                warnings: Some(json!({"warnings":[]})),
                exported_svg: None,
                open_result: None,
                saved_project: None,
                nest_result: Some(
                    json!({"board":{"w":910.0,"h":1820.0},"placements":4,"strategy":"first_fit"}),
                ),
            }
        }
        _ => panic!("unsupported dataset"),
    }
}

>>>>>>> main
#[test]
fn golden_datasets_smoke() {
    let root = repo_root();
    let manifest = load_manifest(&root);
    for ds in &manifest.datasets {
        if !ds.tags.iter().any(|t| t == "smoke") {
            continue;
        }
        let meta = build_meta(ds);
<<<<<<< codex/expand-golden-datasets-for-step-3-thjhms
        let out = run_dataset_by_id(&root, &manifest, &ds.id, NoiseMode::Off).unwrap();
=======
        let out = run_dataset(&root, ds);
>>>>>>> main
        for exp in &ds.expected {
            let ep = root.join(&exp.path);
            match (&exp.kind, &exp.compare) {
                (ExpectedKind::NormalizedModel, CompareMode::JsonStruct) => {
<<<<<<< codex/expand-golden-datasets-for-step-3-thjhms
                    compare_json_struct(&root, &meta, &ep, out.normalized_model.clone()).unwrap()
                }
                (ExpectedKind::Warnings, CompareMode::ReasonCodes) => {
                    compare_reason_codes(&root, &meta, &ep, out.warnings.clone()).unwrap()
=======
                    compare_json_struct(&root, &meta, &ep, out.normalized_model.clone().unwrap())
                        .unwrap()
                }
                (ExpectedKind::Warnings, CompareMode::ReasonCodes) => {
                    compare_reason_codes(&root, &meta, &ep, out.warnings.clone().unwrap()).unwrap()
>>>>>>> main
                }
                (ExpectedKind::ExportedSvg, CompareMode::SvgHash) => {
                    compare_svg_hash(&root, &meta, &ep, &out.exported_svg.clone().unwrap()).unwrap()
                }
<<<<<<< codex/expand-golden-datasets-for-step-3-thjhms
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
=======
                (ExpectedKind::OpenResult, CompareMode::JsonStruct) => {
                    compare_json_struct(&root, &meta, &ep, out.open_result.clone().unwrap())
                        .unwrap()
                }
                (ExpectedKind::SavedProject, CompareMode::JsonStruct) => {
                    compare_json_struct(&root, &meta, &ep, out.saved_project.clone().unwrap())
                        .unwrap()
                }
                (ExpectedKind::NestResult, CompareMode::JsonStruct) => {
                    compare_json_struct(&root, &meta, &ep, out.nest_result.clone().unwrap())
                        .unwrap()
                }
>>>>>>> main
                _ => panic!("unsupported kind/comparison"),
            }
        }
    }
}
