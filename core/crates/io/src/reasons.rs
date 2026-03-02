use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasonCode {
    IoFormatNotRegistered,
    IoLimitBytesExceeded,
    IoSanitizeNonfinite,
    IoPathClosedByEps,
    IoNormalizeRounded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub reason: ReasonCode,
    pub message: String,
    pub hint: Option<String>,
    pub context: BTreeMap<String, String>,
    pub is_fatal: bool,
}

impl AppError {
    pub fn new(reason: ReasonCode, message: impl Into<String>) -> Self {
        Self {
            reason,
            message: message.into(),
            hint: None,
            context: BTreeMap::new(),
            is_fatal: false,
        }
    }

    pub fn fatal(mut self) -> Self {
        self.is_fatal = true;
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_context(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.context.insert(k.into(), v.into());
        self
    }
}

#[derive(Debug, Error)]
#[error("{reason:?}: {message}")]
pub struct AppErrorWrapper {
    pub reason: ReasonCode,
    pub message: String,
    pub hint: Option<String>,
    pub context: BTreeMap<String, String>,
}

impl From<AppError> for AppErrorWrapper {
    fn from(e: AppError) -> Self {
        Self {
            reason: e.reason,
            message: e.message,
            hint: e.hint,
            context: e.context,
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
