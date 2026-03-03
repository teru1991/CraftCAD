#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Render/LOD related stable reasons.

use serde::{Deserialize, Serialize};

/// Stable render reason codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RenderReason {
    /// Rendering degraded due to load/limits; quality reduced but deterministic.
    PerfDegradedMode,
}
