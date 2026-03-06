#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Deterministic batching for renderer backends.

use serde::{Deserialize, Serialize};

use crate::{Primitive, PrimitiveKind, StyleKey};

/// Batching config.
#[derive(Debug, Clone)]
pub struct BatchingConfig {
    /// Max primitives per batch to bound worst-case allocations.
    pub max_per_batch: usize,
}

impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            max_per_batch: 8192,
        }
    }
}

/// Batch grouping key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BatchKey {
    /// Layer draw order.
    pub layer_order: i32,
    /// Style key.
    pub style: StyleKey,
    /// Primitive kind.
    pub kind: PrimitiveKind,
}

/// Output batch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Batch {
    /// Batch key.
    pub key: BatchKey,
    /// Stable item ids.
    pub items: Vec<u64>,
    /// Opaque payloads in same order as `items`.
    pub payloads: Vec<serde_json::Value>,
}

/// Batch already-sorted primitives into stable groups.
pub fn batch_primitives(prims: &[Primitive], cfg: &BatchingConfig) -> Vec<Batch> {
    let mut out: Vec<Batch> = Vec::new();

    let mut cur_key: Option<BatchKey> = None;
    let mut cur_items: Vec<u64> = Vec::new();
    let mut cur_payloads: Vec<serde_json::Value> = Vec::new();

    let flush = |out: &mut Vec<Batch>,
                 key: Option<BatchKey>,
                 items: &mut Vec<u64>,
                 payloads: &mut Vec<serde_json::Value>| {
        if let Some(k) = key {
            if !items.is_empty() {
                out.push(Batch {
                    key: k,
                    items: std::mem::take(items),
                    payloads: std::mem::take(payloads),
                });
            }
        }
    };

    for p in prims {
        let k = BatchKey {
            layer_order: p.layer_order,
            style: p.style.clone(),
            kind: p.kind,
        };
        let need_new = match &cur_key {
            None => true,
            Some(ck) => ck != &k,
        };

        if need_new || cur_items.len() >= cfg.max_per_batch {
            flush(&mut out, cur_key.take(), &mut cur_items, &mut cur_payloads);
            cur_key = Some(k);
        }

        cur_items.push(p.stable_id);
        cur_payloads.push(p.payload.clone());
    }

    flush(&mut out, cur_key.take(), &mut cur_items, &mut cur_payloads);
    out
}
