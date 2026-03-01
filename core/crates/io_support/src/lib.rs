use serde::Deserialize;
use std::collections::BTreeMap;

const SUPPORT_MATRIX: &str = include_str!("../../../../docs/specs/io/support_matrix.json");
const MAPPING_RULES: &str = include_str!("../../../../docs/specs/io/mapping_rules.json");

#[derive(Debug, Clone, Deserialize)]
pub struct SupportCell {
    pub import: SupportLevel,
    pub export: SupportLevel,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SupportMatrix {
    pub version: String,
    pub formats: BTreeMap<String, BTreeMap<String, SupportCell>>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupportLevel {
    Supported,
    BestEffort,
    NotSupported,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Import,
    Export,
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

impl SupportMatrix {
    pub fn load_from_ssot() -> serde_json::Result<Self> {
        serde_json::from_str(SUPPORT_MATRIX)
    }

    pub fn level(&self, format: &str, feature: &str, direction: Direction) -> SupportLevel {
        self.formats
            .get(format)
            .and_then(|m| m.get(feature))
            .map(|cell| match direction {
                Direction::Import => cell.import,
                Direction::Export => cell.export,
            })
            .unwrap_or(SupportLevel::NotSupported)
    }

    pub fn reasons(&self, format: &str, feature: &str, _direction: Direction) -> Vec<String> {
        self.formats
            .get(format)
            .and_then(|m| m.get(feature))
            .map(|cell| cell.reason_codes.clone())
            .unwrap_or_default()
    }
}

impl MappingRules {
    pub fn load() -> serde_json::Result<Self> {
        serde_json::from_str(MAPPING_RULES)
    }
}

pub fn load_support_matrix() -> serde_json::Result<SupportMatrix> {
    SupportMatrix::load_from_ssot()
}

pub fn reason_codes_for(format: &str, feature: &str) -> serde_json::Result<Vec<String>> {
    let matrix = SupportMatrix::load_from_ssot()?;
    Ok(matrix.reasons(format, feature, Direction::Import))
}
