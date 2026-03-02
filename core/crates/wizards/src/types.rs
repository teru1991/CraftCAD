use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardInput {
    pub template_id: String,
    pub inputs: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardResultDraft {
    pub evaluated_ops: Vec<EvaluatedOp>,
    pub seed_used: u64,
    pub template_id: String,
    pub template_version: String,
    pub warnings: Vec<crate::reasons::WizardReason>,
    pub asset_links: Vec<AssetLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLink {
    pub kind: String,
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatedOp {
    pub op: String,
    pub args: BTreeMap<String, serde_json::Value>,
}
