use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub template_id: String,
    pub template_version: String,
    pub schema_version: i32,
    pub kind: String,
    pub display_name_key: Option<String>,
    pub required_presets: RequiredPresets,
    pub ui_inputs: Vec<UiInput>,
    pub generation_steps: Vec<GenStep>,
    pub determinism: Determinism,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredPresets {
    pub material_preset_ids: Vec<String>,
    pub process_preset_ids: Vec<String>,
    pub output_preset_ids: Vec<String>,
    #[serde(default)]
    pub hardware_preset_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Determinism {
    pub seed_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiInput {
    pub key: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub unit: String,
    pub default: serde_json::Value,
    pub min: serde_json::Value,
    pub max: serde_json::Value,
    pub step: serde_json::Value,
    #[serde(default)]
    pub enum_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenStep {
    pub op: String,
    #[serde(default)]
    pub args: BTreeMap<String, serde_json::Value>,
}
