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

    pub fn validate(&self) -> Result<(), String> {
        if !(1..=365).contains(&self.default_keep_days) {
            return Err(format!(
                "default_keep_days out of range: {}",
                self.default_keep_days
            ));
        }
        let min_bytes: u64 = 64 * 1024 * 1024;
        let max_bytes: u64 = 64 * 1024 * 1024 * 1024;
        if !(min_bytes..=max_bytes).contains(&self.max_total_bytes) {
            return Err(format!(
                "max_total_bytes out of range: {}",
                self.max_total_bytes
            ));
        }
        if !(1..=1000).contains(&self.max_items) {
            return Err(format!("max_items out of range: {}", self.max_items));
        }
        Ok(())
    }
}
