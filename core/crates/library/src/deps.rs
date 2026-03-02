use crate::reasons::{LibraryReason, LibraryReasonCode};
use craftcad_presets::model::PresetKind;
use craftcad_presets::resolve::PresetRef;
use craftcad_presets::PresetsService;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RequiredPresets {
    pub material_preset_ids: Vec<String>,
    pub process_preset_ids: Vec<String>,
    pub output_preset_ids: Vec<String>,
    #[serde(default)]
    pub hardware_preset_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WizardTemplateMinimal {
    pub template_id: String,
    pub template_version: String,
    pub schema_version: i32,
    pub required_presets: RequiredPresets,
}

#[derive(Debug, Clone)]
pub struct MissingDeps {
    pub missing: Vec<String>,
}

pub fn resolve_required_presets(
    svc: &PresetsService,
    tpl: &WizardTemplateMinimal,
) -> Result<(), LibraryReason> {
    let mut missing = vec![];
    let req = "^1.0";

    for id in &tpl.required_presets.material_preset_ids {
        let r = PresetRef::parse(PresetKind::Material, id.clone(), req).map_err(|e| {
            LibraryReason::new(
                LibraryReasonCode::LibTemplateInvalid,
                format!("preset ref parse failed: {e:?}"),
            )
        })?;
        if svc.resolve_ref_to_version(&r).is_err() {
            missing.push(id.clone());
        }
    }

    for id in &tpl.required_presets.process_preset_ids {
        let r = PresetRef::parse(PresetKind::Process, id.clone(), req).map_err(|e| {
            LibraryReason::new(
                LibraryReasonCode::LibTemplateInvalid,
                format!("preset ref parse failed: {e:?}"),
            )
        })?;
        if svc.resolve_ref_to_version(&r).is_err() {
            missing.push(id.clone());
        }
    }

    for id in &tpl.required_presets.output_preset_ids {
        let r = PresetRef::parse(PresetKind::Output, id.clone(), req).map_err(|e| {
            LibraryReason::new(
                LibraryReasonCode::LibTemplateInvalid,
                format!("preset ref parse failed: {e:?}"),
            )
        })?;
        if svc.resolve_ref_to_version(&r).is_err() {
            missing.push(id.clone());
        }
    }

    for id in &tpl.required_presets.hardware_preset_ids {
        let r = PresetRef::parse(PresetKind::Hardware, id.clone(), req).map_err(|e| {
            LibraryReason::new(
                LibraryReasonCode::LibTemplateInvalid,
                format!("preset ref parse failed: {e:?}"),
            )
        })?;
        if svc.resolve_ref_to_version(&r).is_err() {
            missing.push(id.clone());
        }
    }

    missing.sort();
    missing.dedup();

    if !missing.is_empty() {
        return Err(LibraryReason::new(
            LibraryReasonCode::LibDepsMissingPreset,
            format!("missing required presets: {}", missing.join(", ")),
        ));
    }

    Ok(())
}
