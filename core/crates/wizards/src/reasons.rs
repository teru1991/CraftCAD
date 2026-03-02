use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WizardReasonCode {
    WizardTemplateInvalid,
    WizardTemplateSchemaInvalid,
    WizardDepMissingPreset,
    WizardInputInvalid,
    WizardDslInvalid,
    WizardDeterminismError,
    WizardIoError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WizardReason {
    pub code: WizardReasonCode,
    pub message: String,
    pub path: Option<String>,
    pub key: Option<String>,
}

impl WizardReason {
    pub fn new(code: WizardReasonCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: None,
            key: None,
        }
    }
    pub fn with_path(mut self, p: impl Into<String>) -> Self {
        self.path = Some(p.into());
        self
    }
    pub fn with_key(mut self, k: impl Into<String>) -> Self {
        self.key = Some(k.into());
        self
    }
}
