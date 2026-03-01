use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpanRecord {
    pub name: String,
    pub duration_ms: f64,
    pub count: u64,
    pub start_order: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerfReport {
    pub dataset_id: String,
    pub schema_version: Option<String>,
    pub seed: Option<u64>,
    pub spans: Vec<SpanRecord>,
    pub memory_peak_mb: Option<u64>,
}

impl PerfReport {
    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}
