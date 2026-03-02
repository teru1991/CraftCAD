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
pub struct LayerRules {
    pub normalize_case: bool,
    pub max_length: usize,
    pub invalid_char_replacement: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinetypeRules {
    pub fallback: String,
    pub supported: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnitRules {
    pub default_import_unit: String,
    pub conversions: BTreeMap<String, f64>,
    pub reason_code_on_assume: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MappingRules {
    pub version: String,
    pub layer_rules: LayerRules,
    pub linetype_rules: LinetypeRules,
    pub unit_rules: UnitRules,
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
            .unwrap_or(SupportLevel::NotSupported)
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
        serde_json::from_str(MAPPING_RULES).map_err(|e| {
            AppError::new(
                ReasonCode::IO_JSON_SCHEMA_INVALID,
                "mapping rules parse error",
            )
            .with_context("error", e.to_string())
            .fatal()
        })
    }

    pub fn map_layer(&self, name: &str) -> String {
        let mut out = if self.layer_rules.normalize_case {
            name.to_uppercase()
        } else {
            name.to_string()
        };
        out = out
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c.to_string()
                } else {
                    self.layer_rules.invalid_char_replacement.clone()
                }
            })
            .collect::<String>();
        out.chars().take(self.layer_rules.max_length).collect()
    }

    pub fn map_linetype(&self, name: &str) -> String {
        let up = name.to_uppercase();
        if self.linetype_rules.supported.iter().any(|x| x == &up) {
            up
        } else {
            self.linetype_rules.fallback.clone()
        }
    }

    pub fn map_units(&self, units: Units) -> Units {
        units
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
