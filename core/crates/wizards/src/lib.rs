pub mod determinism;
pub mod engine;
pub mod reasons;
pub mod template;
pub mod types;

use crate::engine::eval::eval_generation_steps;
use crate::engine::validate::{canonicalize_inputs_json, validate_inputs};
use crate::reasons::{WizardReason, WizardReasonCode};
use crate::template::TemplateRegistry;
use crate::types::{AssetLink, WizardInput, WizardResultDraft};

use craftcad_presets::PresetsService;
use std::path::PathBuf;

pub struct WizardEngine {
    templates: TemplateRegistry,
    presets: PresetsService,
    _repo_root: PathBuf,
    _user_root: PathBuf,
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
            _repo_root: repo_root,
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
}
