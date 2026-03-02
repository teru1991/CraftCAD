use craftcad_io::model::Units;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use regex::Regex;
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
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupportLevel {
    Supported,
    BestEffort,
    NotSupported,
}

#[derive(Debug, Clone, Deserialize)]
struct NormalizeDoc {
    #[serde(default)]
    trim: bool,
    #[serde(default)]
    collapse_whitespace: bool,
    #[serde(default = "default_replace_spaces_with")]
    replace_spaces_with: String,
}

fn default_replace_spaces_with() -> String {
    "_".to_string()
}

#[derive(Debug, Clone, Deserialize)]
struct NamedRulesDoc {
    default: String,
    max_len: usize,
    forbidden_chars_regex: String,
    normalize: NormalizeDoc,
    aliases: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct UnitsRulesDoc {
    supported: Vec<String>,
    default: String,
    #[serde(default)]
    import_guess_order: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExportRulesDoc {
    #[serde(default)]
    decimal_places: u8,
    #[serde(default)]
    force_locale: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MappingRulesDoc {
    schema_version: u32,
    layer: NamedRulesDoc,
    linetype: NamedRulesDoc,
    units: UnitsRulesDoc,
    export: ExportRulesDoc,
}

#[derive(Debug, Clone)]
struct NamedRules {
    default: String,
    max_len: usize,
    forbidden_re: Regex,
    normalize: NormalizeDoc,
    aliases: BTreeMap<String, String>,
}

impl NamedRules {
    fn normalize_and_sanitize(&self, raw: &str) -> String {
        let mut s = raw.to_string();

        if self.normalize.trim {
            s = s.trim().to_string();
        }

        if self.normalize.collapse_whitespace {
            // collapse all consecutive whitespace into a single space
            s = s.split_whitespace().collect::<Vec<_>>().join(" ");
        }

        if self.normalize.replace_spaces_with != " " {
            s = s.replace(' ', &self.normalize.replace_spaces_with);
        }

        // Replace forbidden chars deterministically.
        s = self.forbidden_re.replace_all(&s, "_").to_string();

        // Clamp length (bytes) deterministically.
        if s.len() > self.max_len {
            s.truncate(self.max_len);
        }

        // Normalize into a deterministic key.
        s = s.trim_matches('_').to_string();
        s.to_uppercase()
    }
}

#[derive(Debug, Clone)]
pub struct MappingRules {
    layer: NamedRules,
    linetype: NamedRules,
    unit_default: Units,
    export_decimal_places: u8,
    export_force_locale: String,
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
            .map(|e| e.reason_codes.iter().map(|s| map_reason_code(s)).collect())
            .unwrap_or_else(|| vec![ReasonCode::IO_SUPPORT_MATRIX_FEATURE_MISSING])
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

        if doc.schema_version != 1 {
            return Err(AppError::new(
                ReasonCode::IO_JSON_SCHEMA_UNSUPPORTED_VERSION,
                "unsupported mapping_rules.schema_version",
            )
            .with_context("schema_version", doc.schema_version.to_string())
            .fatal());
        }

        let layer_re = Regex::new(&doc.layer.forbidden_chars_regex).map_err(|e| {
            AppError::new(
                ReasonCode::IO_JSON_SCHEMA_INVALID,
                "layer.forbidden_chars_regex invalid",
            )
            .with_context("error", e.to_string())
            .fatal()
        })?;

        let linetype_re = Regex::new(&doc.linetype.forbidden_chars_regex).map_err(|e| {
            AppError::new(
                ReasonCode::IO_JSON_SCHEMA_INVALID,
                "linetype.forbidden_chars_regex invalid",
            )
            .with_context("error", e.to_string())
            .fatal()
        })?;

        if doc.layer.max_len == 0 || doc.linetype.max_len == 0 {
            return Err(
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "max_len must be > 0").fatal(),
            );
        }

        let unit_default = match doc.units.default.as_str() {
            "mm" => Units::Mm,
            "inch" => Units::Inch,
            other => {
                return Err(AppError::new(
                    ReasonCode::IO_JSON_SCHEMA_INVALID,
                    "units.default must be mm|inch",
                )
                .with_context("units.default", other.to_string())
                .fatal())
            }
        };

        if !doc
            .units
            .supported
            .iter()
            .any(|u| u == doc.units.default.as_str())
        {
            return Err(AppError::new(
                ReasonCode::IO_JSON_SCHEMA_INVALID,
                "units.default must be in units.supported",
            )
            .with_context("units.default", doc.units.default.clone())
            .fatal());
        }

        Ok(Self {
            layer: NamedRules {
                default: doc.layer.default,
                max_len: doc.layer.max_len,
                forbidden_re: layer_re,
                normalize: doc.layer.normalize,
                aliases: doc.layer.aliases,
            },
            linetype: NamedRules {
                default: doc.linetype.default,
                max_len: doc.linetype.max_len,
                forbidden_re: linetype_re,
                normalize: doc.linetype.normalize,
                aliases: doc.linetype.aliases,
            },
            unit_default,
            export_decimal_places: doc.export.decimal_places,
            export_force_locale: doc.export.force_locale,
        })
    }

    /// Layer mapping:
    /// - normalize + sanitize
    /// - apply alias if present
    /// - if empty => default
    /// - else preserve normalized unknown layer name
    pub fn map_layer(&self, name: &str) -> String {
        let key = self.layer.normalize_and_sanitize(name);
        if key.is_empty() {
            return self.layer.default.clone();
        }
        if let Some(v) = self.layer.aliases.get(&key) {
            return v.clone();
        }
        key
    }

    /// Linetype mapping:
    /// - normalize + sanitize
    /// - apply alias if present
    /// - unknown/empty => default (canonical linetypes for interoperability)
    pub fn map_linetype(&self, name: &str) -> String {
        let key = self.linetype.normalize_and_sanitize(name);
        if key.is_empty() {
            return self.linetype.default.clone();
        }
        self.linetype
            .aliases
            .get(&key)
            .cloned()
            .unwrap_or_else(|| self.linetype.default.clone())
    }

    pub fn map_units(&self, units: Units) -> Units {
        match units {
            Units::Mm => Units::Mm,
            Units::Inch => Units::Inch,
        }
    }

    pub fn default_units(&self) -> Units {
        self.unit_default
    }

    pub fn export_decimal_places(&self) -> u8 {
        self.export_decimal_places
    }

    pub fn export_force_locale(&self) -> &str {
        self.export_force_locale.as_str()
    }
}

fn map_reason_code(s: &str) -> ReasonCode {
    match s {
        "IO_CURVE_APPROX_APPLIED" => ReasonCode::IO_CURVE_APPROX_APPLIED,
        "IO_TEXT_FALLBACK_FONT" => ReasonCode::IO_TEXT_FALLBACK_FONT,
        "IO_FALLBACK_024" => ReasonCode::IO_FALLBACK_024,
        "IO_UNSUPPORTED_ENTITY_DXF_SPLINE" => ReasonCode::IO_UNSUPPORTED_ENTITY_DXF_SPLINE,
        "IO_HATCH_SIMPLIFIED" => ReasonCode::IO_HATCH_SIMPLIFIED,
        "IO_IMAGE_REFERENCE_DROPPED" => ReasonCode::IO_IMAGE_REFERENCE_DROPPED,
        _ => ReasonCode::IO_SUPPORT_MATRIX_FEATURE_MISSING,
    }
}
