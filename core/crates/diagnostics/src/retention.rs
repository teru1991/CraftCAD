use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub default_keep_days: u64,
    pub max_total_bytes: u64,
    pub max_items: u64,
}

impl RetentionPolicy {
    pub fn ssot_default() -> Self {
        Self {
            default_keep_days: 14,
            max_total_bytes: 2 * 1024 * 1024 * 1024,
            max_items: 50,
        }
    }
}
