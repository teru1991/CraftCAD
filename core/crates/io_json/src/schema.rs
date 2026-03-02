use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use craftcad_io::report::IoReport;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const SCHEMA_PATH: &str = "docs/specs/io/json_internal.schema.json";

fn schema_abs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(SCHEMA_PATH)
}

fn load_schema() -> Value {
    let p = schema_abs_path();
    let s =
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("failed to read {}: {}", p.display(), e));
    serde_json::from_str(&s)
        .unwrap_or_else(|e| panic!("invalid schema json {}: {}", p.display(), e))
}

pub fn validate_v1(doc: &Value, report: &mut IoReport) -> AppResult<()> {
    let _schema = load_schema();

    let obj = doc.as_object().ok_or_else(|| {
        AppError::new(
            ReasonCode::IO_JSON_SCHEMA_INVALID,
            "json root must be object",
        )
    })?;

    for k in ["schema_version", "units", "entities", "metadata"] {
        if !obj.contains_key(k) {
            return Err(
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "missing required key")
                    .with_context("missing", k),
            );
        }
    }

    let ver = obj
        .get("schema_version")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| {
            AppError::new(
                ReasonCode::IO_JSON_SCHEMA_INVALID,
                "schema_version must be integer",
            )
        })?;
    if ver != 1 {
        return Err(AppError::new(
            ReasonCode::IO_JSON_SCHEMA_UNSUPPORTED_VERSION,
            "unsupported schema_version",
        )
        .with_context("schema_version", ver.to_string())
        .with_hint("対応versionへ変換するか、互換migrationが実装されるまで待ってください。")
        .fatal());
    }

    let units = obj
        .get("units")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "units must be string"))?;
    if units != "mm" && units != "inch" {
        return Err(
            AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "units must be mm|inch")
                .with_context("units", units),
        );
    }

    if obj.get("entities").and_then(|v| v.as_array()).is_none() {
        return Err(AppError::new(
            ReasonCode::IO_JSON_SCHEMA_INVALID,
            "entities must be array",
        ));
    }

    let meta = obj
        .get("metadata")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            AppError::new(
                ReasonCode::IO_JSON_SCHEMA_INVALID,
                "metadata must be object",
            )
        })?;
    for k in ["source_format", "determinism_tag"] {
        if !meta.contains_key(k) {
            return Err(AppError::new(
                ReasonCode::IO_JSON_SCHEMA_INVALID,
                "metadata missing required key",
            )
            .with_context("missing", k));
        }
    }

    for k in obj.keys() {
        if !matches!(
            k.as_str(),
            "schema_version" | "units" | "entities" | "metadata"
        ) {
            return Err(
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "unknown top-level key")
                    .with_context("key", k),
            );
        }
    }

    report.extras.insert(
        "schema_path".into(),
        schema_abs_path().display().to_string(),
    );
    Ok(())
}
