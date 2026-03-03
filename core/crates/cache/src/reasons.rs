#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Cache warnings / reasons (stable identifiers).

use serde::{Deserialize, Serialize};

/// Stable reason code for cache diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CacheReason {
    /// Cache entry rejected due to entry size limit.
    CacheEntryRejectedTooLarge,
    /// Cache entry evicted due to LRU and global cap.
    CacheEvicted,
}

/// Optional structured warning record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheWarning {
    /// Reason code.
    pub reason: CacheReason,
    /// Key identifier.
    pub key: String,
    /// Human-readable message.
    pub message: String,
    /// Optional diagnostic context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

impl CacheWarning {
    /// Create a warning record.
    pub fn new(reason: CacheReason, key: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            reason,
            key: key.into(),
            message: message.into(),
            context: None,
        }
    }

    /// Add structured context.
    pub fn with_context(mut self, ctx: serde_json::Value) -> Self {
        self.context = Some(ctx);
        self
    }
}
