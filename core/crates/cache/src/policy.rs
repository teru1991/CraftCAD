#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Cache policy. Initially constant, later can load from SSOT.

/// Bounded cache capacity policy.
#[derive(Debug, Clone)]
pub struct CachePolicy {
    /// Max total cache bytes across all entries.
    pub max_total_bytes: u64,
    /// Max bytes for a single entry.
    pub max_entry_bytes: u64,
    /// Max number of entries.
    pub max_entries: usize,
}

/// Default policy builder.
pub struct DefaultCachePolicy;

impl DefaultCachePolicy {
    /// Conservative default policy.
    pub fn conservative() -> CachePolicy {
        CachePolicy {
            max_total_bytes: 256 * 1024 * 1024,
            max_entry_bytes: 64 * 1024 * 1024,
            max_entries: 10_000,
        }
    }
}
