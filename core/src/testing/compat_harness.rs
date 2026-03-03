use craftcad_io::options::{ExportOptions, ImportOptions};
use craftcad_io::{Exporter, Importer};
use craftcad_io_bridge::default_engine;
use craftcad_io_dxf::DxfIo;
use craftcad_io_json::JsonIo;
use craftcad_io_svg::SvgIo;
use craftcad_presets::model::PresetKind;
use craftcad_presets::resolve::PresetRef;
use craftcad_presets::PresetsService;
use craftcad_wizards::engine::ast::Template;
use craftcad_wizards::engine::eval::eval_generation_steps;
use craftcad_wizards::engine::validate::canonicalize_inputs_json;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum CompatKind {
    ProjectJson,
    PresetJson,
    TemplateJson,
    IoSvg,
    IoDxfAscii,
}

#[derive(Debug, Clone)]
pub struct CompatCase {
    pub id: &'static str,
    pub kind: CompatKind,
    pub rel_path: &'static str,
}

#[derive(Debug, Clone)]
pub struct CompatError {
    pub code: &'static str,
    pub message: String,
    pub case_id: String,
    pub artifacts_dir: PathBuf,
}

impl fmt::Display for CompatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] compat failed: {} (case_id={}) artifacts_dir={}",
            self.code,
            self.message,
            self.case_id,
            self.artifacts_dir.display()
        )
    }
}
impl std::error::Error for CompatError {}

pub fn artifacts_dir(repo_root: &Path) -> PathBuf {
    if let Ok(p) = std::env::var("CRAFTCAD_FAILURE_ARTIFACTS_DIR") {
        return PathBuf::from(p).join("compat");
    }
    repo_root.join("failure_artifacts").join("compat")
}

fn write_text(path: &Path, bytes: &[u8]) {
    let _ = fs::write(path, bytes);
}

pub fn write_compat_failure_text(
    repo_root: &Path,
    case_id: &str,
    meta: &Value,
    repro_name: &str,
    repro_text_bytes: &[u8],
    actual_json: Option<&Value>,
    diff_text: Option<&str>,
    reason_codes: Option<&[String]>,
) -> PathBuf {
    let base = artifacts_dir(repo_root);
    let dir = base.join(case_id);
    let _ = fs::create_dir_all(&dir);

    let _ = fs::write(
        dir.join("meta.json"),
        serde_json::to_vec_pretty(meta).unwrap_or_else(|_| b"{}".to_vec()),
    );

    write_text(&dir.join(repro_name), repro_text_bytes);

    if let Some(a) = actual_json {
        let _ = fs::write(
            dir.join("actual.json"),
            serde_json::to_vec_pretty(a).unwrap_or_else(|_| b"{}".to_vec()),
        );
    }
    if let Some(d) = diff_text {
        let _ = fs::write(dir.join("diff.txt"), d.as_bytes());
    }
    if let Some(codes) = reason_codes {
        let w = json!({ "codes": codes });
        let _ = fs::write(
            dir.join("reason_codes.json"),
            serde_json::to_vec_pretty(&w).unwrap_or_else(|_| b"{}".to_vec()),
        );
    }
    dir
}

pub fn run_project_json_case(repo_root: &Path, case: &CompatCase) -> Result<(), CompatError> {
    let full = repo_root.join(case.rel_path);
    let input_bytes = fs::read(&full).map_err(|e| CompatError {
        code: "CP_PROJECT_OPEN",
        message: format!("failed to read project json: {}", e),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;

    let input_json: Value = serde_json::from_slice(&input_bytes).map_err(|e| CompatError {
        code: "CP_PROJECT_OPEN",
        message: format!("project json parse error: {}", e),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;

    let schema_v = input_json
        .get("schema_version")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    let current_supported_max: i64 = 1;

    if schema_v > current_supported_max {
        let meta = json!({
            "compat_case_id": case.id,
            "kind": "project_json",
            "path": case.rel_path,
            "schema_version": schema_v,
            "supported_max": current_supported_max,
            "migrate_path": format!("{} -> unsupported", schema_v)
        });
        let dir = write_compat_failure_text(
            repo_root,
            case.id,
            &meta,
            "repro_input.json",
            &input_bytes,
            Some(&input_json),
            Some("forward incompatible schema_version"),
            Some(&["CP_FORWARD_INCOMPATIBLE".to_string()]),
        );
        return Err(CompatError {
            code: "CP_FORWARD_INCOMPATIBLE",
            message: "project schema_version is newer than supported".to_string(),
            case_id: case.id.to_string(),
            artifacts_dir: dir,
        });
    }

    let io = JsonIo::new();
    let iopts = ImportOptions::default_for_tests();
    let imported = io.import_bytes(&input_bytes, &iopts).map_err(|e| {
        let meta = json!({
            "compat_case_id": case.id,
            "kind": "project_json",
            "path": case.rel_path,
            "schema_version": schema_v,
            "migrate_path": format!("{} -> {}", schema_v, current_supported_max)
        });
        let code = vec!["CP_PROJECT_OPEN".to_string()];
        let dir = write_compat_failure_text(
            repo_root,
            case.id,
            &meta,
            "repro_input.json",
            &input_bytes,
            Some(&input_json),
            Some(&format!("json import failed: {}", e.message)),
            Some(&code),
        );
        CompatError {
            code: "CP_PROJECT_OPEN",
            message: format!("json import failed: {}", e.message),
            case_id: case.id.to_string(),
            artifacts_dir: dir,
        }
    })?;

    let eopts = ExportOptions::default_for_tests();
    let exported = io
        .export_bytes(&imported.model, &eopts)
        .map_err(|e| CompatError {
            code: "CP_PROJECT_MIGRATE",
            message: format!("json export failed: {}", e.message),
            case_id: case.id.to_string(),
            artifacts_dir: artifacts_dir(repo_root),
        })?;
    let migrated_json: Value =
        serde_json::from_slice(&exported.bytes).map_err(|e| CompatError {
            code: "CP_PROJECT_MIGRATE",
            message: format!("exported json parse failed: {}", e),
            case_id: case.id.to_string(),
            artifacts_dir: artifacts_dir(repo_root),
        })?;

    if imported.model.entities.is_empty() {
        let meta = json!({
            "compat_case_id": case.id,
            "kind": "project_json",
            "path": case.rel_path,
            "schema_version": schema_v,
            "migrate_path": format!("{} -> {}", schema_v, current_supported_max)
        });
        let dir = write_compat_failure_text(
            repo_root,
            case.id,
            &meta,
            "repro_input.json",
            &input_bytes,
            Some(&migrated_json),
            Some("validate failed: imported model has no entities"),
            Some(&["CP_PROJECT_VALIDATE".to_string()]),
        );
        return Err(CompatError {
            code: "CP_PROJECT_VALIDATE",
            message: "imported model has no entities".to_string(),
            case_id: case.id.to_string(),
            artifacts_dir: dir,
        });
    }

    Ok(())
}

pub fn run_preset_case(repo_root: &Path, case: &CompatCase) -> Result<(), CompatError> {
    let full = repo_root.join(case.rel_path);
    let input_bytes = fs::read(&full).map_err(|e| CompatError {
        code: "CP_PRESET_MIGRATE",
        message: format!("failed to read preset: {}", e),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;
    let input_json: Value = serde_json::from_slice(&input_bytes).map_err(|e| CompatError {
        code: "CP_PRESET_MIGRATE",
        message: format!("preset json parse error: {}", e),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;

    let schema_v = input_json
        .get("schema_version")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    if schema_v < 1 || schema_v > 1 {
        let meta = json!({"compat_case_id":case.id,"kind":"preset","path":case.rel_path,"schema_version":schema_v});
        let dir = write_compat_failure_text(
            repo_root,
            case.id,
            &meta,
            "repro_input.json",
            &input_bytes,
            Some(&input_json),
            Some("unsupported preset schema_version"),
            Some(&["CP_PRESET_MIGRATE".to_string()]),
        );
        return Err(CompatError {
            code: "CP_PRESET_MIGRATE",
            message: "unsupported preset schema_version".to_string(),
            case_id: case.id.to_string(),
            artifacts_dir: dir,
        });
    }

    let id = input_json.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let req = input_json
        .get("version_req")
        .and_then(|v| v.as_str())
        .unwrap_or("^1.0");
    let kind = match input_json
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("")
    {
        "material_preset" => PresetKind::Material,
        "process_preset" => PresetKind::Process,
        "output_preset" => PresetKind::Output,
        "hardware_preset" => PresetKind::Hardware,
        _ => {
            return Err(CompatError {
                code: "CP_PRESET_RESOLVE",
                message: "unknown preset kind".to_string(),
                case_id: case.id.to_string(),
                artifacts_dir: artifacts_dir(repo_root),
            })
        }
    };

    let user_tmp = tempfile::tempdir().map_err(|e| CompatError {
        code: "CP_PRESET_RESOLVE",
        message: format!("tempdir failed: {e}"),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;
    let svc = PresetsService::new(repo_root.to_path_buf(), user_tmp.path().to_path_buf()).map_err(
        |e| CompatError {
            code: "CP_PRESET_RESOLVE",
            message: format!("presets init failed: {e:?}"),
            case_id: case.id.to_string(),
            artifacts_dir: artifacts_dir(repo_root),
        },
    )?;

    let pref = PresetRef::parse(kind, id.to_string(), req).map_err(|e| CompatError {
        code: "CP_PRESET_RESOLVE",
        message: format!("version_req parse failed: {e}"),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;
    svc.resolve_ref_to_version(&pref).map_err(|e| CompatError {
        code: "CP_PRESET_RESOLVE",
        message: format!("preset resolve failed: {e:?}"),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;

    Ok(())
}

pub fn run_template_case(repo_root: &Path, case: &CompatCase) -> Result<(), CompatError> {
    let full = repo_root.join(case.rel_path);
    let input_bytes = fs::read(&full).map_err(|e| CompatError {
        code: "CP_TEMPLATE_VALIDATE",
        message: format!("failed to read template: {}", e),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;
    let input_json: Value = serde_json::from_slice(&input_bytes).map_err(|e| CompatError {
        code: "CP_TEMPLATE_VALIDATE",
        message: format!("template json parse error: {}", e),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;

    let tpl: Template = serde_json::from_value(input_json.clone()).map_err(|e| CompatError {
        code: "CP_TEMPLATE_VALIDATE",
        message: format!("template deserialize failed: {e}"),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;

    let mut defaults = BTreeMap::new();
    for d in &tpl.ui_inputs {
        defaults.insert(d.key.clone(), d.default.clone());
    }
    craftcad_wizards::engine::validate::validate_inputs(&tpl, &defaults).map_err(|e| {
        CompatError {
            code: "CP_TEMPLATE_VALIDATE",
            message: format!("template inputs validation failed: {}", e.message),
            case_id: case.id.to_string(),
            artifacts_dir: artifacts_dir(repo_root),
        }
    })?;

    let canon = canonicalize_inputs_json(&tpl, &defaults).map_err(|e| CompatError {
        code: "CP_TEMPLATE_EXECUTE",
        message: format!("canonicalize inputs failed: {}", e.message),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;
    let filled: BTreeMap<String, Value> =
        serde_json::from_str(&canon).map_err(|e| CompatError {
            code: "CP_TEMPLATE_EXECUTE",
            message: format!("canonical inputs parse failed: {e}"),
            case_id: case.id.to_string(),
            artifacts_dir: artifacts_dir(repo_root),
        })?;

    eval_generation_steps(&tpl, &filled).map_err(|e| CompatError {
        code: "CP_TEMPLATE_EXECUTE",
        message: format!("template dry-run execute failed: {}", e.message),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;

    Ok(())
}

pub fn run_io_svg_case(repo_root: &Path, case: &CompatCase) -> Result<(), CompatError> {
    let full = repo_root.join(case.rel_path);
    let input = fs::read(&full).map_err(|e| CompatError {
        code: "CP_IO_IMPORT",
        message: format!("failed to read svg: {}", e),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;

    let io = SvgIo::new();
    let iopts = ImportOptions::default_for_tests();
    let imported = io.import_bytes(&input, &iopts).map_err(|e| {
        let meta = json!({"compat_case_id":case.id,"kind":"io_svg","path":case.rel_path});
        let dir = write_compat_failure_text(
            repo_root,
            case.id,
            &meta,
            "repro_input.svg",
            &input,
            None,
            Some(&format!("svg import failed: {}", e.message)),
            Some(&["CP_IO_IMPORT".to_string()]),
        );
        CompatError {
            code: "CP_IO_IMPORT",
            message: format!("svg import failed: {}", e.message),
            case_id: case.id.to_string(),
            artifacts_dir: dir,
        }
    })?;

    let eng = default_engine();
    let _normalized = eng
        .export("json", &imported.model, &ExportOptions::default_for_tests())
        .map_err(|e| CompatError {
            code: "CP_IO_IMPORT",
            message: format!("svg normalize export failed: {}", e.message),
            case_id: case.id.to_string(),
            artifacts_dir: artifacts_dir(repo_root),
        })?;

    Ok(())
}

pub fn run_io_dxf_ascii_case(repo_root: &Path, case: &CompatCase) -> Result<(), CompatError> {
    let full = repo_root.join(case.rel_path);
    let input = fs::read(&full).map_err(|e| CompatError {
        code: "CP_IO_IMPORT",
        message: format!("failed to read dxf: {}", e),
        case_id: case.id.to_string(),
        artifacts_dir: artifacts_dir(repo_root),
    })?;

    let s = String::from_utf8(input.clone()).map_err(|e| {
        let meta =
            json!({ "compat_case_id": case.id, "kind":"io_dxf_ascii", "path": case.rel_path });
        let dir = write_compat_failure_text(
            repo_root,
            case.id,
            &meta,
            "repro_input.txt",
            &input,
            None,
            Some("DXF is not valid UTF-8 => binary DXF is forbidden in Step4"),
            Some(&["CP_IO_IMPORT".to_string()]),
        );
        CompatError {
            code: "CP_IO_IMPORT",
            message: format!("DXF is not UTF-8 (binary forbidden): {}", e),
            case_id: case.id.to_string(),
            artifacts_dir: dir,
        }
    })?;

    if !(s.contains("SECTION") && s.contains("ENTITIES")) {
        let meta =
            json!({ "compat_case_id": case.id, "kind":"io_dxf_ascii", "path": case.rel_path });
        let dir = write_compat_failure_text(
            repo_root,
            case.id,
            &meta,
            "repro_input.txt",
            s.as_bytes(),
            None,
            Some("ASCII DXF missing SECTION/ENTITIES"),
            Some(&["CP_IO_IMPORT".to_string()]),
        );
        return Err(CompatError {
            code: "CP_IO_IMPORT",
            message: "ASCII DXF missing SECTION/ENTITIES".to_string(),
            case_id: case.id.to_string(),
            artifacts_dir: dir,
        });
    }

    let io = DxfIo::new();
    io.import_bytes(s.as_bytes(), &ImportOptions::default_for_tests())
        .map_err(|e| CompatError {
            code: "CP_IO_IMPORT",
            message: format!("dxf import failed: {}", e.message),
            case_id: case.id.to_string(),
            artifacts_dir: artifacts_dir(repo_root),
        })?;

    Ok(())
}
