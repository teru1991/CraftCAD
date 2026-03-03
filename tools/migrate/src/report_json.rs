use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    pub input: String,
    pub output: Option<String>,
    pub ok: bool,
    pub reason: Option<String>,
    pub warnings: Vec<String>,
    pub migrate_overall_from: Option<i64>,
    pub migrate_overall_to: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    pub version: String,
    pub totals: BTreeMap<String, u64>,
    pub files: Vec<FileSummary>,
}

pub fn stable_json_value(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut new = serde_json::Map::new();
            for k in keys {
                new.insert(k.clone(), stable_json_value(&map[&k]));
            }
            serde_json::Value::Object(new)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(stable_json_value).collect())
        }
        _ => v.clone(),
    }
}
