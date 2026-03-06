#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Thread-safe cache store with epoch-based LRU eviction.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

use crate::key::CacheKey;
use crate::policy::CachePolicy;
use crate::reasons::{CacheReason, CacheWarning};
use crate::{CacheError, CacheResult};

/// Cached value payload with size estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Estimated bytes for bounded cache accounting.
    pub bytes: u64,
    /// Cached value.
    pub value: serde_json::Value,
}

/// Cache statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Current live entry count.
    pub entries: usize,
    /// Current total bytes.
    pub total_bytes: u64,
    /// Total cache hits.
    pub hits: u64,
    /// Total cache misses.
    pub misses: u64,
    /// Total evicted entries.
    pub evictions: u64,
}

#[derive(Debug)]
struct EntryMeta {
    entry: CacheEntry,
    last_used_epoch: u64,
}

#[derive(Debug)]
struct Inner {
    policy: CachePolicy,
    epoch: u64,
    total_bytes: u64,
    map: HashMap<String, EntryMeta>,
    epochs: BTreeMap<u64, Vec<String>>,
    stats: CacheStats,
    warnings: Vec<CacheWarning>,
}

/// Thread-safe cache store.
pub struct CacheStore {
    inner: Mutex<Inner>,
}

impl CacheStore {
    /// Create a new cache store.
    pub fn new(policy: CachePolicy) -> Self {
        Self {
            inner: Mutex::new(Inner {
                policy,
                epoch: 0,
                total_bytes: 0,
                map: HashMap::new(),
                epochs: BTreeMap::new(),
                stats: CacheStats::default(),
                warnings: Vec::new(),
            }),
        }
    }

    fn lock(&self) -> CacheResult<MutexGuard<'_, Inner>> {
        self.inner.lock().map_err(|_| CacheError::Poisoned)
    }

    fn next_epoch_locked(g: &mut Inner) -> u64 {
        if let Some(v) = g.epoch.checked_add(1) {
            g.epoch = v;
            return v;
        }
        Self::compact_epochs_locked(g);
        g.epoch = 1;
        1
    }

    /// Returns and clears accumulated warnings in deterministic order.
    pub fn drain_warnings(&self) -> CacheResult<Vec<CacheWarning>> {
        let mut g = self.lock()?;
        g.warnings
            .sort_by(|a, b| a.key.cmp(&b.key).then(a.message.cmp(&b.message)));
        Ok(std::mem::take(&mut g.warnings))
    }

    /// Return current stats snapshot.
    pub fn stats(&self) -> CacheResult<CacheStats> {
        let g = self.lock()?;
        Ok(g.stats.clone())
    }

    /// Get cached value and update LRU state if present.
    pub fn get(&self, key: &CacheKey) -> CacheResult<Option<serde_json::Value>> {
        let mut g = self.lock()?;
        let k = key.as_str().to_string();

        if g.map.contains_key(&k) {
            let epoch = Self::next_epoch_locked(&mut g);
            let value = {
                let meta = g.map.get_mut(&k).expect("checked");
                meta.last_used_epoch = epoch;
                meta.entry.value.clone()
            };
            g.epochs.entry(epoch).or_default().push(k);
            g.stats.hits += 1;
            return Ok(Some(value));
        }

        g.stats.misses += 1;
        Ok(None)
    }

    /// Insert/update entry and evict by policy if necessary.
    pub fn put(&self, key: &CacheKey, entry: CacheEntry) -> CacheResult<()> {
        let mut g = self.lock()?;

        if entry.bytes > g.policy.max_entry_bytes {
            let max_entry_bytes = g.policy.max_entry_bytes;
            g.warnings.push(
                CacheWarning::new(
                    CacheReason::CacheEntryRejectedTooLarge,
                    format!("key:{}", key.as_str()),
                    "entry rejected: too large",
                )
                .with_context(serde_json::json!({
                    "entry_bytes": entry.bytes,
                    "max_entry_bytes": max_entry_bytes
                })),
            );
            return Ok(());
        }

        let k = key.as_str().to_string();

        if let Some(old) = g.map.remove(&k) {
            g.total_bytes = g.total_bytes.saturating_sub(old.entry.bytes);
        }

        let epoch = Self::next_epoch_locked(&mut g);

        g.total_bytes = g.total_bytes.saturating_add(entry.bytes);
        g.map.insert(
            k.clone(),
            EntryMeta {
                entry,
                last_used_epoch: epoch,
            },
        );
        g.epochs.entry(epoch).or_default().push(k);

        Self::evict_if_needed_locked(&mut g);

        g.stats.entries = g.map.len();
        g.stats.total_bytes = g.total_bytes;

        Ok(())
    }

    fn compact_epochs_locked(g: &mut Inner) {
        g.epochs.clear();
        for (k, meta) in &g.map {
            g.epochs
                .entry(meta.last_used_epoch)
                .or_default()
                .push(k.clone());
        }
    }

    fn evict_if_needed_locked(g: &mut Inner) {
        while (g.total_bytes > g.policy.max_total_bytes) || (g.map.len() > g.policy.max_entries) {
            let Some((&oldest_epoch, _)) = g.epochs.iter().next() else {
                break;
            };

            let k = {
                let keys = g.epochs.get_mut(&oldest_epoch).expect("exists");
                let popped = keys.pop();
                let empty = keys.is_empty();
                if empty {
                    g.epochs.remove(&oldest_epoch);
                }
                match popped {
                    Some(v) => v,
                    None => continue,
                }
            };

            let remove = match g.map.get(&k) {
                Some(meta) => meta.last_used_epoch == oldest_epoch,
                None => false,
            };
            if !remove {
                continue;
            }

            if let Some(meta) = g.map.remove(&k) {
                g.total_bytes = g.total_bytes.saturating_sub(meta.entry.bytes);
                g.stats.evictions += 1;
                let max_total_bytes = g.policy.max_total_bytes;
                g.warnings.push(
                    CacheWarning::new(
                        CacheReason::CacheEvicted,
                        format!("key:{k}"),
                        "entry evicted",
                    )
                    .with_context(serde_json::json!({
                        "entry_bytes": meta.entry.bytes,
                        "total_bytes": g.total_bytes,
                        "max_total_bytes": max_total_bytes
                    })),
                );
            }
        }
    }

    /// Clears all entries.
    pub fn clear(&self) -> CacheResult<()> {
        let mut g = self.lock()?;
        g.map.clear();
        g.epochs.clear();
        g.total_bytes = 0;
        g.stats.entries = 0;
        g.stats.total_bytes = 0;
        Ok(())
    }
}
