#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Deterministic cache core for CraftCAD.
//!
//! Key goals:
//! - Deterministic CacheKey (canonical JSON hashing, no map-order dependency)
//! - Bounded memory usage (bytes caps) with silent eviction
//! - Safe invalidation via (ssot_hash, schema_version, inputs hash) encoded in key

mod key;
mod policy;
mod reasons;
mod store;

pub use key::{CacheKey, CacheKeyMaterial, CanonicalJson, Sha256Hex};
pub use policy::{CachePolicy, DefaultCachePolicy};
pub use reasons::{CacheReason, CacheWarning};
pub use store::{CacheEntry, CacheStats, CacheStore};

/// Cache result alias.
pub type CacheResult<T> = Result<T, CacheError>;

/// Cache-level error category.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// Invalid key material.
    #[error("invalid key material: {0}")]
    InvalidKey(String),
    /// Entry exceeded configured entry size cap.
    #[error("entry too large: {0}")]
    EntryTooLarge(String),
    /// Internal mutex poisoned.
    #[error("store is poisoned")]
    Poisoned,
}
