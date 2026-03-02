use craftcad_io::model::Units;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use serde::Deserialize;
use std::collections::BTreeMap;

const SUPPORT_MATRIX: &str = include_str!("../../../../docs/specs/io/support_matrix.json");
const MAPPING_RULES: &str = include_str!("../../../../docs/specs/io/mapping_rules.json");

#[derive(Debug, Clone, Deserialize)]
struct SupportMatrixDoc {
    matrix: Vec<SupportEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct SupportEntry {
    format: String,
    direction: String,
    feature: String,
    level: SupportLevel,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupportLevel {
    Supported,
    BestEffort,
    NotSupported,
}

#[derive(Debug, Clone, Deserialize)]
struct NamedRules {
    default: String,
    aliases: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct UnitsRules {
    default: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MappingRulesDoc {
    layer: NamedRules,
    linetype: NamedRules,
    units: UnitsRules,
}

#[derive(Debug, Clone)]
pub struct MappingRules {
    layer_default: String,
    layer_aliases: BTreeMap<String, String>,
    linetype_default: String,
    linetype_aliases: BTreeMap<String, String>,
    unit_default: String,
}

#[derive(Debug, Clone)]
pub struct SupportMatrix {
    entries: Vec<SupportEntry>,
}

impl SupportMatrix {
    pub fn load_from_ssot() -> AppResult<Self> {
        let doc: SupportMatrixDoc = serde_json::from_str(SUPPORT_MATRIX).map_err(|e| {
            AppError::new(
                ReasonCode::IO_JSON_SCHEMA_INVALID,
                "support matrix parse error",
            )
            .with_context("error", e.to_string())
            .fatal()
        })?;
        Ok(Self {
            entries: doc.matrix,
        })
    }

    pub fn level(&self, format: &str, feature: &str, direction: &str) -> SupportLevel {
        self.entries
            .iter()
            .find(|e| e.format == format && e.feature == feature && e.direction == direction)
            .map(|e| e.level)
            .unwrap_or(SupportLevel::Supported)
    }

    pub fn reasons(&self, format: &str, feature: &str, direction: &str) -> Vec<ReasonCode> {
        self.entries
            .iter()
            .find(|e| e.format == format && e.feature == feature && e.direction == direction)
            .map(|e| {
                e.reason_codes
                    .iter()
                    .map(|s| map_reason_code(s))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    pub fn action(&self, format: &str, feature: &str, direction: &str) -> Option<String> {
        self.entries
            .iter()
            .find(|e| e.format == format && e.feature == feature && e.direction == direction)
            .and_then(|e| e.action.clone())
    }
}

impl MappingRules {
    pub fn load_from_ssot() -> AppResult<Self> {
        let doc: MappingRulesDoc = serde_json::from_str(MAPPING_RULES).map_err(|e| {
            AppError::new(
                ReasonCode::IO_JSON_SCHEMA_INVALID,
                "mapping rules parse error",
            )
            .with_context("error", e.to_string())
            .fatal()
        })?;
        Ok(Self {
            layer_default: doc.layer.default,
            layer_aliases: doc.layer.aliases,
            linetype_default: doc.linetype.default,
            linetype_aliases: doc.linetype.aliases,
            unit_default: doc.units.default,
        })
    }

    pub fn map_layer(&self, name: &str) -> String {
        let n = name.trim().replace(' ', "_").to_uppercase();
        self.layer_aliases
            .get(&n)
            .cloned()
            .unwrap_or_else(|| self.layer_default.clone())
    }

    pub fn map_linetype(&self, name: &str) -> String {
        let n = name.trim().replace(' ', "_").to_uppercase();
        self.linetype_aliases
            .get(&n)
            .cloned()
            .unwrap_or_else(|| self.linetype_default.clone())
    }

    pub fn map_units(&self, units: Units) -> Units {
        match units {
            Units::Mm => Units::Mm,
            Units::Inch => Units::Inch,
        }
    }

    pub fn default_units(&self) -> Units {
        if self.unit_default == "inch" {
            Units::Inch
        } else {
            Units::Mm
        }
    }
}

fn map_reason_code(s: &str) -> ReasonCode {
    match s {
        "IO_CURVE_APPROX_APPLIED" => ReasonCode::IO_CURVE_APPROX_APPLIED,
        "IO_TEXT_FALLBACK_FONT" => ReasonCode::IO_NORMALIZE_ROUNDED,
        "IO_FALLBACK_024" => ReasonCode::IO_NORMALIZE_ROUNDED,
        "IO_UNSUPPORTED_ENTITY_DXF_SPLINE" => ReasonCode::IO_DXF_ENTITY_UNKNOWN_DROPPED,
        "IO_HATCH_SIMPLIFIED" => ReasonCode::IO_DXF_ENTITY_UNKNOWN_DROPPED,
        "IO_IMAGE_REFERENCE_DROPPED" => ReasonCode::IO_SVG_EXTERNAL_REFERENCE_BLOCKED,
        _ => ReasonCode::IO_NORMALIZE_ROUNDED,
    }
}
