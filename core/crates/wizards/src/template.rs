use crate::engine::ast::Template;
use crate::reasons::{WizardReason, WizardReasonCode};
use craftcad_presets::PresetsService;
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root_from_manifest() -> PathBuf {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for up in 0..=10usize {
        let mut p = start.clone();
        for _ in 0..up {
            p = p.parent().unwrap_or(&p).to_path_buf();
        }
        if p.join("docs").join("specs").exists() {
            return p;
        }
    }
    panic!("repo root not found from {}", start.display());
}

fn read_json(path: &Path) -> Result<Value, WizardReason> {
    let s = fs::read_to_string(path).map_err(|e| {
        WizardReason::new(WizardReasonCode::WizardIoError, format!("read failed: {e}"))
            .with_path(path.display().to_string())
    })?;
    serde_json::from_str(&s).map_err(|e| {
        WizardReason::new(
            WizardReasonCode::WizardTemplateInvalid,
            format!("json parse failed: {e}"),
        )
        .with_path(path.display().to_string())
    })
}

fn compile_schema(schema_path: &Path) -> Result<JSONSchema, WizardReason> {
    let v = read_json(schema_path)?;
    JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&v)
        .map_err(|e| {
            WizardReason::new(
                WizardReasonCode::WizardTemplateSchemaInvalid,
                format!("schema compile failed: {e}"),
            )
            .with_path(schema_path.display().to_string())
        })
}

pub struct TemplateRegistry {
    repo_root: PathBuf,
    compiled_schema: JSONSchema,
}

impl TemplateRegistry {
    pub fn new(repo_root: Option<PathBuf>) -> Result<Self, WizardReason> {
        let root = repo_root.unwrap_or_else(repo_root_from_manifest);
        let dir = root.join("docs").join("specs").join("templates");
        let schema_path = dir.join("wizard_template.schema.json");
        let compiled = compile_schema(&schema_path)?;
        Ok(Self {
            repo_root: root,
            compiled_schema: compiled,
        })
    }

    pub fn load_builtin_template(&self, template_file: &str) -> Result<Template, WizardReason> {
        let dir = self.repo_root.join("docs").join("specs").join("templates");
        let p = dir.join(template_file);
        let v = read_json(&p)?;
        if let Err(errors) = self.compiled_schema.validate(&v) {
            let mut msg = String::new();
            for e in errors {
                msg.push_str(&format!("- {e} at {}\n", e.instance_path));
            }
            return Err(WizardReason::new(
                WizardReasonCode::WizardTemplateSchemaInvalid,
                format!("template schema invalid:\n{msg}"),
            )
            .with_path(p.display().to_string()));
        }
        serde_json::from_value::<Template>(v).map_err(|e| {
            WizardReason::new(
                WizardReasonCode::WizardTemplateInvalid,
                format!("template deserialize failed: {e}"),
            )
            .with_path(p.display().to_string())
        })
    }

    pub fn verify_required_presets_exist(
        &self,
        presets: &PresetsService,
        tpl: &Template,
    ) -> Result<(), WizardReason> {
        let req = "^1.0";
        for id in &tpl.required_presets.material_preset_ids {
            let r = craftcad_presets::resolve::PresetRef::parse(
                craftcad_presets::model::PresetKind::Material,
                id.clone(),
                req,
            )
            .map_err(|e| {
                WizardReason::new(
                    WizardReasonCode::WizardTemplateInvalid,
                    format!("preset ref parse failed: {e:?}"),
                )
            })?;
            if presets.resolve_ref_to_version(&r).is_err() {
                return Err(WizardReason::new(
                    WizardReasonCode::WizardDepMissingPreset,
                    format!("missing required material preset: {id}"),
                ));
            }
        }
        for id in &tpl.required_presets.process_preset_ids {
            let r = craftcad_presets::resolve::PresetRef::parse(
                craftcad_presets::model::PresetKind::Process,
                id.clone(),
                req,
            )
            .map_err(|e| {
                WizardReason::new(
                    WizardReasonCode::WizardTemplateInvalid,
                    format!("preset ref parse failed: {e:?}"),
                )
            })?;
            if presets.resolve_ref_to_version(&r).is_err() {
                return Err(WizardReason::new(
                    WizardReasonCode::WizardDepMissingPreset,
                    format!("missing required process preset: {id}"),
                ));
            }
        }
        for id in &tpl.required_presets.output_preset_ids {
            let r = craftcad_presets::resolve::PresetRef::parse(
                craftcad_presets::model::PresetKind::Output,
                id.clone(),
                req,
            )
            .map_err(|e| {
                WizardReason::new(
                    WizardReasonCode::WizardTemplateInvalid,
                    format!("preset ref parse failed: {e:?}"),
                )
            })?;
            if presets.resolve_ref_to_version(&r).is_err() {
                return Err(WizardReason::new(
                    WizardReasonCode::WizardDepMissingPreset,
                    format!("missing required output preset: {id}"),
                ));
            }
        }
        for id in &tpl.required_presets.hardware_preset_ids {
            let r = craftcad_presets::resolve::PresetRef::parse(
                craftcad_presets::model::PresetKind::Hardware,
                id.clone(),
                req,
            )
            .map_err(|e| {
                WizardReason::new(
                    WizardReasonCode::WizardTemplateInvalid,
                    format!("preset ref parse failed: {e:?}"),
                )
            })?;
            if presets.resolve_ref_to_version(&r).is_err() {
                return Err(WizardReason::new(
                    WizardReasonCode::WizardDepMissingPreset,
                    format!("missing required hardware preset: {id}"),
                ));
            }
        }
        Ok(())
    }
}
