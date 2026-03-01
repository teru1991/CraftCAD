use serde::Deserialize;
use std::collections::BTreeMap;

const SUPPORT_MATRIX: &str = include_str!("../../../../docs/specs/io/support_matrix.json");

#[derive(Debug, Clone, Deserialize)]
pub struct SupportCell {
    pub import: String,
    pub export: String,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SupportMatrix {
    pub version: String,
    pub formats: BTreeMap<String, BTreeMap<String, SupportCell>>,
}

pub fn load_support_matrix() -> serde_json::Result<SupportMatrix> {
    serde_json::from_str(SUPPORT_MATRIX)
}

pub fn reason_codes_for(format: &str, feature: &str) -> serde_json::Result<Vec<String>> {
    let matrix = load_support_matrix()?;
    Ok(matrix
        .formats
        .get(format)
        .and_then(|m| m.get(feature))
        .map(|c| c.reason_codes.clone())
        .unwrap_or_default())
}
