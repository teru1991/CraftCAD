use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionBump {
    pub from: i64,
    pub to: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeSet {
    pub added: BTreeSet<String>,
    pub removed: BTreeSet<String>,
    pub changed: BTreeSet<String>,
    pub notes: Vec<String>,
}

impl ChangeSet {
    pub fn add_added(&mut self, p: impl Into<String>) {
        self.added.insert(p.into());
    }

    pub fn add_removed(&mut self, p: impl Into<String>) {
        self.removed.insert(p.into());
    }

    pub fn add_changed(&mut self, p: impl Into<String>) {
        self.changed.insert(p.into());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountDelta {
    pub parts: i64,
    pub nest_jobs: i64,
    pub assets: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepReport {
    pub bump: VersionBump,
    pub changes: ChangeSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateReport {
    pub overall_from: i64,
    pub overall_to: i64,
    pub steps: Vec<StepReport>,
    pub count_delta: CountDelta,
}
