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
