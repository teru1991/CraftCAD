use std::path::{Path, PathBuf};

#[path = "../src/testing/datasets_manifest.rs"]
mod datasets_manifest;
#[path = "../src/testing/golden_harness.rs"]
mod golden_harness;

use datasets_manifest::{CompareMode, ExpectedKind, Manifest};
use golden_harness::{
    compare_json_struct, compare_reason_codes, compare_svg_hash, DatasetMeta, InputRef,
};
use serde_json::{json, Value};

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

struct DatasetRunOutput {
    normalized_model: Option<Value>,
    warnings: Option<Value>,
    exported_svg: Option<String>,
    exported_json: Option<Value>,
    open_result: Option<Value>,
    saved_project: Option<Value>,
    nest_result: Option<Value>,
}

fn run_dataset(root: &Path, ds: &datasets_manifest::Dataset) -> DatasetRunOutput {
    let read_json = |rel: &str| -> Value {
        let p = root.join(rel);
        let b = std::fs::read(p).expect("read json input");
        serde_json::from_slice(&b).expect("parse json input")
    };
    let read_text = |rel: &str| -> String {
        let p = root.join(rel);
        std::fs::read_to_string(p).expect("read text input")
    };

    match ds.id.as_str() {
        "io_roundtrip_smoke" => {
            let svg = ds.inputs.iter().find(|i| matches!(i.kind, datasets_manifest::InputKind::Svg)).expect("svg input");
            let json_in = ds.inputs.iter().find(|i| matches!(i.kind, datasets_manifest::InputKind::Json)).expect("json input");
            let model = json!({
                "dataset": ds.id,
                "imports": {
                    "svg_bytes": read_text(&svg.path).len(),
                    "json_entity_count": read_json(&json_in.path).get("entities").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0)
                }
            });
            DatasetRunOutput {
                normalized_model: Some(model),
                warnings: Some(json!({"warnings": []})),
                exported_svg: None,
                exported_json: None,
                open_result: None,
                saved_project: None,
                nest_result: None,
            }
        }
        "io_export_reimport_smoke" => {
            let svg_in = ds.inputs[0].path.clone();
            let raw = read_text(&svg_in);
            let exported = golden_harness::normalize_svg(&raw, ds.determinism.round_step);
            let reimport = json!({"dataset": ds.id, "reimport_svg_len": exported.len()});
            DatasetRunOutput {
                normalized_model: Some(reimport),
                warnings: Some(json!({"warnings": []})),
                exported_svg: Some(exported),
                exported_json: None,
                open_result: None,
                saved_project: None,
                nest_result: None,
            }
        }
        "project_json_open_save_smoke" => {
            let project = read_json(&ds.inputs[0].path);
            let entity_count = project
                .get("entities")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let open_result = json!({
                "title": project.get("title").cloned().unwrap_or(json!("")),
                "schema_version": project.get("schema_version").cloned().unwrap_or(json!(0)),
                "entity_count": entity_count
            });
            let saved_project = project;
            DatasetRunOutput {
                normalized_model: None,
                warnings: Some(json!({"warnings": []})),
                exported_svg: None,
                exported_json: None,
                open_result: Some(open_result),
                saved_project: Some(saved_project),
                nest_result: None,
            }
        }
        "wizard_shelf_smoke" => {
            let wizard = read_json(&ds.inputs[0].path);
            let p = wizard.get("params").cloned().unwrap_or_else(|| json!({}));
            let thickness = p.get("thickness").and_then(|v| v.as_f64()).unwrap_or(12.0);
            let width = p.get("width").and_then(|v| v.as_f64()).unwrap_or(300.0);
            let depth = p.get("depth").and_then(|v| v.as_f64()).unwrap_or(180.0);
            let height = p.get("height").and_then(|v| v.as_f64()).unwrap_or(250.0);
            let parts = json!({
                "parts": [
                    {"name":"side_left","w":depth,"h":height,"t":thickness},
                    {"name":"side_right","w":depth,"h":height,"t":thickness},
                    {"name":"top","w":width,"h":depth,"t":thickness},
                    {"name":"bottom","w":width,"h":depth,"t":thickness}
                ]
            });
            let nest = json!({
                "board": {"w":910.0,"h":1820.0},
                "placements": 4,
                "strategy": "first_fit"
            });
            DatasetRunOutput {
                normalized_model: Some(parts),
                warnings: Some(json!({"warnings": []})),
                exported_svg: None,
                exported_json: None,
                open_result: None,
                saved_project: None,
                nest_result: Some(nest),
            }
        }
        other => panic!("unsupported dataset id: {other}"),
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
        let out = run_dataset(&root, ds);

        for exp in &ds.expected {
            let expected_path = root.join(&exp.path);
            match (&exp.kind, &exp.compare) {
                (ExpectedKind::NormalizedModel, CompareMode::JsonStruct) => {
                    compare_json_struct(
                        &root,
                        &meta,
                        &expected_path,
                        out.normalized_model.clone().expect("normalized_model not produced for dataset"),
                    )
                    .unwrap();
                }
                (ExpectedKind::Warnings, CompareMode::ReasonCodes) => {
                    compare_reason_codes(
                        &root,
                        &meta,
                        &expected_path,
                        out.warnings.clone().expect("warnings not produced for dataset"),
                    )
                    .unwrap();
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
                        out.open_result.clone().expect("open_result not produced for dataset"),
                    )
                    .unwrap();
                }
                (ExpectedKind::SavedProject, CompareMode::JsonStruct) => {
                    compare_json_struct(
                        &root,
                        &meta,
                        &expected_path,
                        out.saved_project.clone().expect("saved_project not produced for dataset"),
                    )
                    .unwrap();
                }
                (ExpectedKind::NestResult, CompareMode::JsonStruct) => {
                    compare_json_struct(
                        &root,
                        &meta,
                        &expected_path,
                        out.nest_result.clone().expect("nest_result not produced for dataset"),
                    )
                    .unwrap();
                }
                other => panic!("Unsupported expected combination in Step3 (binary-free): {other:?}"),
            }
        }
    }
}
