#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Job trait and result types.

use serde::{Deserialize, Serialize};

use crate::cancel::{CancelToken, Cancelled};
use crate::progress::ProgressReporter;
use crate::reasons::{JobReason, JobWarning};

/// Stable job id (string to allow app-specific scheme).
pub type JobId = String;

/// Logical kind of job (used for routing / policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobKind {
    /// Import pipeline work.
    IoImport,
    /// Export pipeline work.
    IoExport,
    /// Nesting optimizer work.
    Nesting,
    /// Render rebuild / caching work.
    RenderRebuild,
    /// Open project/document.
    Open,
    /// Save project/document.
    Save,
    /// Other queueable work.
    Other,
}

/// Priority class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobPriority {
    /// Highest priority.
    High,
    /// Default priority.
    Normal,
    /// Lowest priority.
    Low,
}

/// Job output is opaque JSON for cross-FFI friendliness.
pub type JobOutput = serde_json::Value;

/// Job result; always includes a stable reason on failure/cancel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// True when job completed successfully.
    pub ok: bool,
    /// Stable reason code for non-ok outcomes.
    pub reason: Option<JobReason>,
    /// Human readable message.
    pub message: String,
    /// Optional output on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<JobOutput>,
    /// Non-fatal warnings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<JobWarning>,
}

impl JobResult {
    /// Success constructor.
    pub fn success(output: JobOutput) -> Self {
        Self {
            ok: true,
            reason: None,
            message: String::new(),
            output: Some(output),
            warnings: vec![],
        }
    }

    /// Cooperative cancellation constructor.
    pub fn cancelled() -> Self {
        Self {
            ok: false,
            reason: Some(JobReason::JobCancelled),
            message: "cancelled".to_string(),
            output: None,
            warnings: vec![],
        }
    }

    /// Failure constructor.
    pub fn failed(reason: JobReason, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            reason: Some(reason),
            message: message.into(),
            output: None,
            warnings: vec![],
        }
    }
}

/// Job trait. Implementations should be panic-free and cooperative-cancel.
pub trait Job: Send + 'static {
    /// Stable id.
    fn id(&self) -> JobId;
    /// Logical job kind.
    fn kind(&self) -> JobKind;
    /// Priority class.
    fn priority(&self) -> JobPriority;

    /// Run the job. Must periodically call `cancel.check()`.
    fn run(
        &mut self,
        cancel: CancelToken,
        progress: ProgressReporter,
    ) -> Result<JobOutput, JobCancelledOrError>;
}

/// Unified error for job execution.
#[derive(Debug, thiserror::Error)]
pub enum JobCancelledOrError {
    /// Cooperative cancellation.
    #[error("cancelled")]
    Cancelled,
    /// Domain failure.
    #[error("failed: {0}")]
    Failed(String),
}

impl From<Cancelled> for JobCancelledOrError {
    fn from(_: Cancelled) -> Self {
        JobCancelledOrError::Cancelled
    }
}
