use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
use jsonschema::JSONSchema;
use serde_json::Value;
use std::collections::BTreeMap;

const MANIFEST_SCHEMA: &str =
    include_str!("../../../../docs/specs/schema/diycad/manifest.schema.json");
const DOCUMENT_SCHEMA: &str =
    include_str!("../../../../docs/specs/schema/diycad/document.schema.json");
const PART_SCHEMA: &str = include_str!("../../../../docs/specs/schema/diycad/part.schema.json");
const NEST_JOB_SCHEMA: &str =
    include_str!("../../../../docs/specs/schema/diycad/nest_job.schema.json");

// ===== Salvage Related Types =====

#[derive(Debug, Clone)]
pub struct SalvageReport {
    pub recovered: bool,
    pub issues: Vec<(String, String)>,  // (field, issue)
    pub normalized_fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FailedPart {
    pub filename: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct FailedNestJob {
    pub filename: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct SalvageResult {
    pub document_recovered: bool,
    pub parts_loaded: usize,
    pub parts_failed: Vec<FailedPart>,
    pub nest_jobs_loaded: usize,
    pub nest_jobs_failed: Vec<FailedNestJob>,
    pub warnings: Vec<(String, ReasonCode)>,
    pub read_only: bool,
}

// ===== Validation Functions =====

pub fn validate_manifest(v: &Value) -> AppResult<()> {
    validate_with_schema(MANIFEST_SCHEMA, v, "manifest")
}

pub fn validate_document(v: &Value) -> AppResult<()> {
    validate_with_schema(DOCUMENT_SCHEMA, v, "document")
}

pub fn validate_part(v: &Value) -> AppResult<()> {
    validate_with_schema(PART_SCHEMA, v, "part")
}

pub fn validate_nest_job(v: &Value) -> AppResult<()> {
    validate_with_schema(NEST_JOB_SCHEMA, v, "nest_job")
}

// ===== Salvage Functions =====

/// Attempt to salvage a document JSON that failed schema validation.
/// Applies best-effort normalization (filling missing optional fields with defaults).
pub fn salvage_document(document_value: &Value) -> AppResult<(Value, SalvageReport)> {
    let mut report = SalvageReport {
        recovered: false,
        issues: Vec::new(),
        normalized_fields: Vec::new(),
    };

    if !document_value.is_object() {
        report.issues.push(("root".to_string(), "not a JSON object".to_string()));
        return Err(AppError::new(
            ReasonCode::new("SALVAGE_DOCUMENT_MALFORMED"),
            Severity::Error,
            "document is not a JSON object",
        ));
    }

    let mut doc = document_value.clone();
    let obj = doc.as_object_mut().unwrap();

    // Check for critical required fields from document.schema.json
    let required_fields = vec!["id", "schema_version", "created_by", "updated_at", "title", "parts", "nest_jobs", "settings"];
    let mut missing_required = Vec::new();

    for field in &required_fields {
        if !obj.contains_key(*field) {
            missing_required.push(*field);
            report.issues.push((field.to_string(), "missing required field".to_string()));
        }
    }

    // If critical fields are missing, cannot salvage
    if !missing_required.is_empty() {
        return Err(AppError::new(
            ReasonCode::new("SALVAGE_DOCUMENT_MALFORMED"),
            Severity::Error,
            format!("document missing required fields: {:?}", missing_required),
        ));
    }

    // Re-validate after checking critical fields
    match validate_document(&doc) {
        Ok(_) => {
            report.recovered = true;
            Ok((doc, report))
        }
        Err(e) => {
            report.issues.push(("schema".to_string(), e.message.clone()));
            Err(AppError::new(
                ReasonCode::new("SALVAGE_DOCUMENT_MALFORMED"),
                Severity::Error,
                "document recovery failed after validation",
            )
            .with_context("details", e.message))
        }
    }
}

pub fn summarize_zip_entries<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            out.insert(file.name().to_string(), file.size());
        }
    }
    out
}

fn validate_with_schema(schema_src: &str, v: &Value, kind: &str) -> AppResult<()> {
    let schema_json: Value = serde_json::from_str(schema_src).map_err(|e| {
        AppError::new(
            ReasonCode::new("SAVE_SCHEMA_INVALID"),
            Severity::Fatal,
            format!("schema parse failed for {kind}: {e}"),
        )
    })?;
    let compiled = JSONSchema::compile(&schema_json).map_err(|e| {
        AppError::new(
            ReasonCode::new("SAVE_SCHEMA_INVALID"),
            Severity::Fatal,
            format!("schema compile failed for {kind}: {e}"),
        )
    })?;

    if let Err(errors) = compiled.validate(v) {
        let detail = errors.map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Err(AppError::new(
            ReasonCode::new("SAVE_SCHEMA_VALIDATION_FAILED"),
            Severity::Error,
            format!("{kind} validation failed"),
        )
        .with_hint("ReadOnlyで開くか、diycad-migrate --dry-run で救出可能性を確認してください")
        .with_context("validation", detail));
    }
    Ok(())
}
