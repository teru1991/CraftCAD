use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLogContext {
    pub app_version: String,
    pub schema_version: String,
    pub os: String,
    pub locale: String,
    pub dataset_id: Option<String>,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStep {
    pub timestamp_ms: u64,
    pub action_id: String,
    pub params_hash: String,
    pub result: String,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLog {
    pub context: JobLogContext,
    pub steps: Vec<JobStep>,
}

impl JobLog {
    pub fn new(context: JobLogContext) -> Self {
        Self {
            context,
            steps: vec![],
        }
    }

    pub fn push_step(&mut self, step: JobStep) {
        self.steps.push(step);
    }

    pub fn finish(self) -> Self {
        self
    }
}
