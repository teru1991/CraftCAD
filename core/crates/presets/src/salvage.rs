use crate::model::PresetKind;
use crate::reasons::{PresetReason, PresetReasonCode};
use crate::PresetsService;
use semver::{Version, VersionReq};

#[derive(Debug, Clone)]
pub struct SalvageSuggestion {
    pub missing_kind: String,
    pub missing_id: String,
    pub requested_version: String,
    pub fallback_builtin_versions: Vec<String>,
}

impl PresetsService {
    pub fn salvage_missing_preset(
        &self,
        kind: PresetKind,
        id: &str,
        version_req: &str,
    ) -> Result<SalvageSuggestion, PresetReason> {
        let req = VersionReq::parse(version_req).map_err(|e| {
            PresetReason::new(
                PresetReasonCode::PresetSemverInvalid,
                format!("version req invalid: {version_req} ({e})"),
            )
        })?;

        let kind_key = match kind {
            PresetKind::Material => "material",
            PresetKind::Process => "process",
            PresetKind::Output => "output",
            PresetKind::Hardware => "hardware",
        };

        let mut versions: Vec<Version> = self
            .items
            .get(&(id.to_string(), kind_key))
            .cloned()
            .unwrap_or_default();
        versions.sort();
        versions.dedup();

        let matched: Vec<String> = versions
            .into_iter()
            .filter(|v| req.matches(v))
            .map(|v| v.to_string())
            .collect();

        Ok(SalvageSuggestion {
            missing_kind: kind_key.to_string(),
            missing_id: id.to_string(),
            requested_version: version_req.to_string(),
            fallback_builtin_versions: matched,
        })
    }
}
