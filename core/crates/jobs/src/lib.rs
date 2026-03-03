#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! CraftCAD job queue core (background execution to keep UI responsive).
//!
//! Guarantees:
//! - Priority: High > Normal > Low
//! - Determinism: FIFO by submit order within same priority
//! - Cancellation: cancel is normal termination (JOB_CANCELLED)
//! - Progress: monotonic 0..1

mod cancel;
mod job;
mod progress;
mod queue;
mod reasons;

pub use cancel::{CancelToken, Cancelled};
pub use job::{Job, JobCancelledOrError, JobId, JobKind, JobOutput, JobPriority, JobResult};
pub use progress::{ProgressReporter, ProgressSnapshot};
pub use queue::{JobHandle, JobQueue, JobQueueConfig};
pub use reasons::{JobReason, JobWarning};

/// Crate result.
pub type JobCoreResult<T> = Result<T, JobCoreError>;

/// Top-level queue/core errors.
#[derive(Debug, thiserror::Error)]
pub enum JobCoreError {
    /// Queue is shut down and no longer accepts work.
    #[error("queue is shut down")]
    Shutdown,
    /// Queue capacity exceeded.
    #[error("queue is full")]
    QueueFull,
    /// Invalid runtime configuration.
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}
