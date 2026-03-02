use crate::ReasonCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: i64,
    pub app_version: String,
    pub created_at: String,
    pub updated_at: String,
    pub unit: Unit,
    pub entrypoints: Entrypoints,

    #[serde(default)]
    pub features: Option<std::collections::BTreeMap<String, bool>>,

    #[serde(default)]
    pub determinism_tag: Option<DeterminismTag>,

    #[serde(default)]
    pub content_manifest: Option<ContentManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entrypoints {
    pub document: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Mm,
    Inch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismTag {
    #[serde(default)]
    pub seed: Option<i64>,
    #[serde(default)]
    pub eps: Option<f64>,
    #[serde(default)]
    pub rounding_decimals: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentManifest {
    pub entries: Vec<ContentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEntry {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub unit: Unit,
    pub entities: Vec<serde_json::Value>,
    pub parts_index: Vec<String>,
    pub nest_jobs_index: Vec<String>,

    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub quantity: i64,
    pub material: serde_json::Value,
    pub geometry: serde_json::Value,

    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestJob {
    pub id: String,
    pub status: String,
    pub inputs: serde_json::Value,

    #[serde(default)]
    pub results: Option<serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WarningKind {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct AppWarning {
    pub code: ReasonCode,
    pub path: Option<String>,
    pub kind: WarningKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct FailedEntry {
    pub path: String,
    pub code: ReasonCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SalvageActionHint {
    ExportSalvagedParts,
    ExportSalvagedDocument,
    GenerateDiagnosticsZip,
    ResaveAsNewProject,
    SuggestMigrateTool,
}

#[derive(Debug, Clone)]
pub struct OpenResult {
    pub read_only: bool,
    pub manifest: Option<Manifest>,
    pub document: Document,
    pub parts_loaded: Vec<Part>,
    pub parts_failed: Vec<FailedEntry>,
    pub nest_jobs_loaded: Vec<NestJob>,
    pub nest_jobs_failed: Vec<FailedEntry>,
    pub warnings: Vec<AppWarning>,
    pub salvage_actions: Vec<SalvageActionHint>,
    pub migrate_report: Option<migration::MigrateReport>,
}

#[derive(Debug, Clone)]
pub struct OpenOptions {
    pub allow_salvage: bool,
    pub verify_integrity: bool,
    pub allow_forward_compat_readonly: bool,
    pub strict_schema: bool,
    pub limits: crate::Limits,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            allow_salvage: true,
            verify_integrity: true,
            allow_forward_compat_readonly: true,
            strict_schema: false,
            limits: crate::Limits::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaveOptions {
    pub atomic: bool,
    pub write_content_manifest: bool,
    pub include_assets: bool,
    pub validate_before_save: bool,
    pub normalize_before_save: bool,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            atomic: true,
            write_content_manifest: true,
            include_assets: true,
            validate_before_save: true,
            normalize_before_save: true,
        }
    }
}
