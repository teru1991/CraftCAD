pub mod r#box;
pub mod determinism;
pub mod engine;
pub mod leather_pouch;
pub mod parts;
pub mod reasons;
pub mod shelf;
pub mod template;
pub mod types;

use crate::engine::eval::eval_generation_steps;
use crate::engine::validate::{canonicalize_inputs_json, validate_inputs};
use crate::parts::model::PartsDraft;
use crate::reasons::{WizardReason, WizardReasonCode};
use crate::template::TemplateRegistry;
use crate::types::{AssetLink, WizardInput, WizardResultDraft};

use craftcad_presets::PresetsService;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub struct WizardEngine {
    templates: TemplateRegistry,
    presets: PresetsService,
    repo_root: PathBuf,
    _user_root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct BundleDoc {
    materials: Vec<MaterialDoc>,
    processes: Vec<ProcessDoc>,
}

#[derive(Debug, Deserialize)]
struct MaterialDoc {
    id: String,
    grain: String,
}

#[derive(Debug, Deserialize)]
struct ProcessDoc {
    id: String,
    kerf_mm: f64,
    margin_mm: f64,
}

impl WizardEngine {
    pub fn new(repo_root: PathBuf, user_root: PathBuf) -> Result<Self, WizardReason> {
        let templates = TemplateRegistry::new(Some(repo_root.clone()))?;
        let presets = PresetsService::new(repo_root.clone(), user_root.clone()).map_err(|e| {
            WizardReason::new(
                WizardReasonCode::WizardIoError,
                format!("presets init failed: {e:?}"),
            )
        })?;
        Ok(Self {
            templates,
            presets,
            repo_root,
            _user_root: user_root,
        })
    }

    pub fn run_template_draft(
        &self,
        template_file: &str,
        input: &WizardInput,
    ) -> Result<WizardResultDraft, WizardReason> {
        let tpl = self.templates.load_builtin_template(template_file)?;
        if tpl.template_id != input.template_id {
            return Err(WizardReason::new(
                WizardReasonCode::WizardInputInvalid,
                format!(
                    "template_id mismatch: input={}, tpl={}",
                    input.template_id, tpl.template_id
                ),
            ));
        }

        self.templates
            .verify_required_presets_exist(&self.presets, &tpl)?;

        validate_inputs(&tpl, &input.inputs)?;
        let canon = canonicalize_inputs_json(&tpl, &input.inputs)?;
        let derived = crate::determinism::derive_seed_from_template_and_inputs(
            &tpl.template_id,
            &tpl.template_version,
            &canon,
        );
        let seed_used = crate::determinism::choose_seed(input.seed, derived)?;

        let filled: std::collections::BTreeMap<String, serde_json::Value> =
            serde_json::from_str(&canon).map_err(|e| {
                WizardReason::new(
                    WizardReasonCode::WizardDeterminismError,
                    format!("filled parse failed: {e}"),
                )
            })?;

        let evaluated = eval_generation_steps(&tpl, &filled)?;

        let mut asset_links = vec![AssetLink {
            kind: "template".into(),
            id: tpl.template_id.clone(),
            version: tpl.template_version.clone(),
        }];
        for id in &tpl.required_presets.material_preset_ids {
            asset_links.push(AssetLink {
                kind: "preset".into(),
                id: id.clone(),
                version: "^1.0".into(),
            });
        }
        for id in &tpl.required_presets.process_preset_ids {
            asset_links.push(AssetLink {
                kind: "preset".into(),
                id: id.clone(),
                version: "^1.0".into(),
            });
        }
        for id in &tpl.required_presets.output_preset_ids {
            asset_links.push(AssetLink {
                kind: "preset".into(),
                id: id.clone(),
                version: "^1.0".into(),
            });
        }
        for id in &tpl.required_presets.hardware_preset_ids {
            asset_links.push(AssetLink {
                kind: "preset".into(),
                id: id.clone(),
                version: "^1.0".into(),
            });
        }

        Ok(WizardResultDraft {
            evaluated_ops: evaluated,
            seed_used,
            template_id: tpl.template_id,
            template_version: tpl.template_version,
            warnings: vec![],
            asset_links,
        })
    }

    pub fn run_wizard_parts(
        &self,
        template_file: &str,
        input: &WizardInput,
    ) -> Result<PartsDraft, WizardReason> {
        let draft = self.run_template_draft(template_file, input)?;
        let tpl = self.templates.load_builtin_template(template_file)?;

        let material_id = tpl
            .required_presets
            .material_preset_ids
            .first()
            .cloned()
            .ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardDepMissingPreset,
                    "no material preset required",
                )
            })?;
        let process_id = tpl
            .required_presets
            .process_preset_ids
            .first()
            .cloned()
            .ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardDepMissingPreset,
                    "no process preset required",
                )
            })?;
        let _output_id = tpl
            .required_presets
            .output_preset_ids
            .first()
            .cloned()
            .ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardDepMissingPreset,
                    "no output preset required",
                )
            })?;

        let bundle_path = self
            .repo_root
            .join("docs")
            .join("specs")
            .join("presets")
            .join("built_in_presets.json");
        let raw = fs::read_to_string(&bundle_path).map_err(|e| {
            WizardReason::new(WizardReasonCode::WizardIoError, format!("read failed: {e}"))
                .with_path(bundle_path.display().to_string())
        })?;
        let bundle: BundleDoc = serde_json::from_str(&raw).map_err(|e| {
            WizardReason::new(
                WizardReasonCode::WizardTemplateInvalid,
                format!("bundle parse failed: {e}"),
            )
            .with_path(bundle_path.display().to_string())
        })?;

        let mat = bundle
            .materials
            .iter()
            .find(|m| m.id == material_id)
            .ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardDepMissingPreset,
                    "material preset not found",
                )
            })?;
        let proc = bundle
            .processes
            .iter()
            .find(|p| p.id == process_id)
            .ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardDepMissingPreset,
                    "process preset not found",
                )
            })?;

        let grain = match mat.grain.as_str() {
            "dir_x" => "dir_x",
            "dir_y" => "dir_y",
            _ => "none",
        }
        .to_string();

        match tpl.kind.as_str() {
            "shelf" => {
                let q = input
                    .inputs
                    .get("quantity")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1) as i32;
                crate::shelf::build_from_evaluated(
                    &tpl.template_id,
                    draft.seed_used,
                    &draft.evaluated_ops,
                    &material_id,
                    &process_id,
                    proc.kerf_mm,
                    proc.margin_mm,
                    &grain,
                    q,
                )
            }
            "box" => crate::r#box::build_from_evaluated(
                &tpl.template_id,
                draft.seed_used,
                &draft.evaluated_ops,
                &material_id,
                &process_id,
                proc.kerf_mm,
                proc.margin_mm,
                &grain,
            ),
            "leather_pouch" => {
                let seam = input
                    .inputs
                    .get("seam_allowance_mm")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(4.0);
                let pitch = input
                    .inputs
                    .get("hole_pitch_mm")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(4.0);
                crate::leather_pouch::build_from_evaluated(
                    &tpl.template_id,
                    draft.seed_used,
                    &draft.evaluated_ops,
                    &material_id,
                    &process_id,
                    proc.kerf_mm,
                    proc.margin_mm,
                    &grain,
                    seam,
                    pitch,
                )
            }
            other => Err(WizardReason::new(
                WizardReasonCode::WizardTemplateInvalid,
                format!("unknown template kind: {other}"),
            )),
        }
    }
}
