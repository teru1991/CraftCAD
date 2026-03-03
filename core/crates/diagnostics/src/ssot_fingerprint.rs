use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SsotFingerprint {
    pub items: Vec<SsotFingerprintItem>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SsotFingerprintItem {
    pub path: String,
    pub sha256: String,
}

impl SsotFingerprint {
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            warnings: Vec::new(),
        }
    }
}
