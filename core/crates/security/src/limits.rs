use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityLimits {
    pub max_import_bytes: u64,
    pub max_entities: u64,
    pub max_zip_entries: u64,
    pub max_json_depth: u64,
    pub max_string_len: u64,
}

pub fn load_limits(path: impl AsRef<Path>) -> Result<SecurityLimits, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}
