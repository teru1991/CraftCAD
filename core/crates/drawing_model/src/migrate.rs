use crate::model::DrawingDoc;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DrawingMigrateError {
    #[error("drawing doc must be a JSON object")]
    NotObject,

    #[error("missing schema_version")]
    MissingSchemaVersion,

    #[error("unsupported schema_version: {0}")]
    UnsupportedSchemaVersion(u32),

    #[error("migration failed: {0}")]
    Failed(String),
}

pub fn migrate_json_to_latest(mut v: Value) -> Result<Value, DrawingMigrateError> {
    let obj = v.as_object_mut().ok_or(DrawingMigrateError::NotObject)?;

    let sv = obj
        .get("schema_version")
        .and_then(|x| x.as_u64())
        .ok_or(DrawingMigrateError::MissingSchemaVersion)? as u32;

    match sv {
        1 => Ok(v),
        _ => Err(DrawingMigrateError::UnsupportedSchemaVersion(sv)),
    }
}

pub fn migrate_and_parse_latest(v: Value) -> Result<DrawingDoc, DrawingMigrateError> {
    let v = migrate_json_to_latest(v)?;
    serde_json::from_value::<DrawingDoc>(v).map_err(|e| DrawingMigrateError::Failed(e.to_string()))
}
