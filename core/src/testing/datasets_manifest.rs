use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub datasets: Vec<Dataset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub id: String,
    pub description: String,
    pub seed: u64,
    pub determinism: Determinism,
    pub limits_ref: String,
    pub inputs: Vec<InputAsset>,
    pub expected: Vec<ExpectedAsset>,
    #[serde(default)]
    pub perf_budget: Option<PerfBudget>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Determinism {
    pub epsilon: f64,
    pub round_step: f64,
    pub ordering_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfBudget {
    #[serde(default)]
    pub max_ms: Option<u64>,
    #[serde(default)]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAsset {
    pub kind: InputKind,
    pub path: String,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedAsset {
    pub kind: ExpectedKind,
    pub path: String,
    pub compare: CompareMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputKind {
    Diycad,
    Dxf,
    Svg,
    Json,
    Preset,
    Template,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedKind {
    NormalizedModel,
    Warnings,
    ExportedSvg,
    ExportedDxf,
    ExportedJson,
    DrawingSvg,
    NestResult,
    OpenResult,
    SavedProject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompareMode {
    JsonStruct,
    SvgHash,
    BytesHash,
    ReasonCodes,
}

#[derive(Debug, Clone)]
pub struct DatasetsManifestError {
    pub code: &'static str,
    pub message: String,
    pub field: Option<String>,
    pub dataset_id: Option<String>,
}

impl DatasetsManifestError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            field: None,
            dataset_id: None,
        }
    }

    fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    fn with_dataset(mut self, dataset_id: impl Into<String>) -> Self {
        self.dataset_id = Some(dataset_id.into());
        self
    }
}

impl fmt::Display for DatasetsManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(dataset_id) = &self.dataset_id {
            write!(f, " (dataset_id={})", dataset_id)?;
        }
        if let Some(field) = &self.field {
            write!(f, " (field={})", field)?;
        }
        Ok(())
    }
}

impl std::error::Error for DatasetsManifestError {}

pub fn validate_manifest(
    manifest: &Manifest,
    repo_root: &Path,
) -> Result<(), DatasetsManifestError> {
    if manifest.version.trim().is_empty() {
        return Err(
            DatasetsManifestError::new("DS_MANIFEST_INVALID", "version must be non-empty")
                .with_field("version"),
        );
    }
    if manifest.datasets.is_empty() {
        return Err(DatasetsManifestError::new(
            "DS_MANIFEST_INVALID",
            "datasets must be non-empty",
        )
        .with_field("datasets"));
    }

    for (idx, dataset) in manifest.datasets.iter().enumerate() {
        validate_dataset(dataset, idx, repo_root)?;
    }

    Ok(())
}

fn validate_dataset(
    ds: &Dataset,
    idx: usize,
    repo_root: &Path,
) -> Result<(), DatasetsManifestError> {
    if !is_valid_dataset_id(&ds.id) {
        return Err(
            DatasetsManifestError::new("DS_MANIFEST_INVALID", "invalid dataset id format")
                .with_dataset(ds.id.clone())
                .with_field(format!("datasets[{idx}].id")),
        );
    }
    if ds.description.trim().is_empty() {
        return Err(DatasetsManifestError::new(
            "DS_MANIFEST_INVALID",
            "description must be non-empty",
        )
        .with_dataset(ds.id.clone())
        .with_field(format!("datasets[{idx}].description")));
    }
    if ds.limits_ref.trim().is_empty() {
        return Err(DatasetsManifestError::new(
            "DS_MANIFEST_INVALID",
            "limits_ref must be non-empty",
        )
        .with_dataset(ds.id.clone())
        .with_field(format!("datasets[{idx}].limits_ref")));
    }

    validate_determinism(&ds.determinism).map_err(|e| {
        e.with_dataset(ds.id.clone())
            .with_field(format!("datasets[{idx}].determinism"))
    })?;

    if ds.inputs.is_empty() {
        return Err(
            DatasetsManifestError::new("DS_MANIFEST_INVALID", "inputs must be non-empty")
                .with_dataset(ds.id.clone())
                .with_field(format!("datasets[{idx}].inputs")),
        );
    }
    if ds.expected.is_empty() {
        return Err(DatasetsManifestError::new(
            "DS_MANIFEST_INVALID",
            "expected must be non-empty",
        )
        .with_dataset(ds.id.clone())
        .with_field(format!("datasets[{idx}].expected")));
    }

    for (j, input) in ds.inputs.iter().enumerate() {
        validate_input_asset(input, repo_root).map_err(|e| {
            e.with_dataset(ds.id.clone())
                .with_field(format!("datasets[{idx}].inputs[{j}]"))
        })?;
    }

    for (j, expected) in ds.expected.iter().enumerate() {
        validate_expected_asset(expected, repo_root).map_err(|e| {
            e.with_dataset(ds.id.clone())
                .with_field(format!("datasets[{idx}].expected[{j}]"))
        })?;
    }

    for (j, tag) in ds.tags.iter().enumerate() {
        if !is_valid_tag(tag) {
            return Err(
                DatasetsManifestError::new("DS_MANIFEST_INVALID", "invalid tag format")
                    .with_dataset(ds.id.clone())
                    .with_field(format!("datasets[{idx}].tags[{j}]")),
            );
        }
    }

    Ok(())
}

fn validate_determinism(determinism: &Determinism) -> Result<(), DatasetsManifestError> {
    if !(determinism.epsilon.is_finite()
        && determinism.epsilon > 0.0
        && determinism.epsilon <= 1e-2)
    {
        return Err(DatasetsManifestError::new(
            "DS_MANIFEST_RANGE_INVALID",
            "epsilon must be (0, 1e-2]",
        )
        .with_field("determinism.epsilon"));
    }
    if !(determinism.round_step.is_finite()
        && determinism.round_step > 0.0
        && determinism.round_step <= 1e-2)
    {
        return Err(DatasetsManifestError::new(
            "DS_MANIFEST_RANGE_INVALID",
            "round_step must be (0, 1e-2]",
        )
        .with_field("determinism.round_step"));
    }
    if determinism.ordering_tag.trim().is_empty() {
        return Err(DatasetsManifestError::new(
            "DS_MANIFEST_INVALID",
            "ordering_tag must be non-empty",
        )
        .with_field("determinism.ordering_tag"));
    }

    Ok(())
}

fn validate_input_asset(input: &InputAsset, repo_root: &Path) -> Result<(), DatasetsManifestError> {
    ensure_under(&input.path, "tests/golden/inputs/")?;
    ensure_path_exists(repo_root, &input.path)?;

    if let Some(hash) = &input.sha256 {
        if !is_lower_hex_64(hash) {
            return Err(DatasetsManifestError::new(
                "DS_MANIFEST_HASH_INVALID",
                "sha256 must be 64 lowercase hex chars",
            )
            .with_field("inputs[].sha256"));
        }
    }

    Ok(())
}

fn validate_expected_asset(
    expected: &ExpectedAsset,
    repo_root: &Path,
) -> Result<(), DatasetsManifestError> {
    ensure_under(&expected.path, "tests/golden/expected/")?;
    ensure_path_exists(repo_root, &expected.path)?;
    Ok(())
}

fn ensure_under(rel: &str, allowed_prefix: &str) -> Result<(), DatasetsManifestError> {
    if !is_safe_rel_path(rel) {
        return Err(DatasetsManifestError::new(
            "DS_MANIFEST_PATH_TRAVERSAL",
            "path must be safe relative (no abs/..)",
        )
        .with_field("path"));
    }

    if !rel.starts_with(allowed_prefix) {
        return Err(DatasetsManifestError::new(
            "DS_MANIFEST_INVALID",
            format!("path must start with {allowed_prefix}"),
        )
        .with_field("path"));
    }

    Ok(())
}

fn ensure_path_exists(repo_root: &Path, rel: &str) -> Result<(), DatasetsManifestError> {
    let full_path: PathBuf = repo_root.join(rel);
    if !full_path.exists() {
        return Err(DatasetsManifestError::new(
            "DS_MANIFEST_INVALID",
            format!("referenced path does not exist: {rel}"),
        )
        .with_field("path"));
    }

    Ok(())
}

pub fn is_safe_rel_path(rel: &str) -> bool {
    if rel.trim().is_empty() {
        return false;
    }
    if rel.contains('\\') {
        return false;
    }
    if rel.starts_with('/') {
        return false;
    }
    for segment in rel.split('/') {
        if segment == ".." || segment.is_empty() {
            return false;
        }
    }
    true
}

fn is_lower_hex_64(value: &str) -> bool {
    if value.len() != 64 {
        return false;
    }
    value
        .bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn is_valid_dataset_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 3 || bytes.len() > 64 {
        return false;
    }
    if !matches!(bytes[0], b'a'..=b'z') {
        return false;
    }
    bytes
        .iter()
        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-'))
}

fn is_valid_tag(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > 24 {
        return false;
    }
    bytes
        .iter()
        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_rel_path_rejects_abs_and_dotdot() {
        assert!(!is_safe_rel_path("/etc/passwd"));
        assert!(!is_safe_rel_path("tests/../secrets.txt"));
        assert!(!is_safe_rel_path("tests\\golden\\inputs\\x"));
        assert!(!is_safe_rel_path("tests/golden//inputs/x"));
        assert!(is_safe_rel_path("tests/golden/inputs/x.txt"));
    }

    #[test]
    fn test_hash_validation() {
        assert!(is_lower_hex_64(&"a".repeat(64)));
        assert!(!is_lower_hex_64(&"A".repeat(64)));
        assert!(!is_lower_hex_64(&"a".repeat(63)));
    }

    #[test]
    fn test_compare_enum_validation() {
        let parsed: CompareMode = serde_json::from_str("\"json_struct\"").expect("must parse");
        assert!(matches!(parsed, CompareMode::JsonStruct));

        let invalid = serde_json::from_str::<CompareMode>("\"bad_compare\"");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_id_regex() {
        assert!(is_valid_dataset_id("io_roundtrip_smoke"));
        assert!(!is_valid_dataset_id("IO_roundtrip_smoke"));
        assert!(!is_valid_dataset_id("a"));
        assert!(!is_valid_dataset_id("a.."));
    }
}
