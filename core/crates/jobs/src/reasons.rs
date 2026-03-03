#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Reason codes for job failures (stable identifiers).

use serde::{Deserialize, Serialize};

/// Stable reason code set for queue/job outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobReason {
    /// Job was cancelled by user/system.
    JobCancelled,
    /// Job panicked; panic was caught and converted to a failure.
    JobPanicked,
    /// Queue was full and job could not be accepted.
    JobQueueFull,
    /// Queue was shut down before / during execution.
    JobQueueShutdown,
    /// Job returned an error (domain-specific; context should explain).
    JobFailed,
}

/// Non-fatal warning emitted by queue/job runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobWarning {
    /// Warning reason code.
    pub reason: JobReason,
    /// Stable warning key for matching in tests/UX.
    pub key: String,
    /// Human readable message.
    pub message: String,
    /// Optional structured context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

impl JobWarning {
    /// Construct a warning.
    pub fn new(reason: JobReason, key: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            reason,
            key: key.into(),
            message: message.into(),
            context: None,
        }
    }

    /// Attach structured context.
    pub fn with_context(mut self, ctx: serde_json::Value) -> Self {
        self.context = Some(ctx);
        self
    }
}
