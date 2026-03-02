use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPolicy {
    pub autosave_interval_sec: u64,
    pub max_generations: usize,
    pub max_total_bytes: u64,
    pub keep_last_good: bool,
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        Self {
            autosave_interval_sec: 60,
            max_generations: 20,
            max_total_bytes: 2 * 1024 * 1024 * 1024,
            keep_last_good: true,
        }
    }
}
