use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryReasonCode {
    LibTagInvalid,
    LibTagNormalized,
    LibIndexCorrupt,
    LibIndexRebuilt,
    LibDepsMissingPreset,
    LibTemplateInvalid,
    LibIoError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryReason {
    pub code: LibraryReasonCode,
    pub message: String,
    pub path: Option<String>,
    pub key: Option<String>,
}

impl LibraryReason {
    pub fn new(code: LibraryReasonCode, message: impl Into<String>) -> Self {
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
