#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! CacheKey with deterministic (canonical) hashing.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{CacheError, CacheResult};

/// Hex string of SHA-256.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Sha256Hex(pub String);

impl Sha256Hex {
    /// Compute SHA-256 hex from bytes.
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b);
        let out = hasher.finalize();
        Sha256Hex(hex::encode(out))
    }

    /// Compute SHA-256 hex from a JSON value after canonicalization.
    pub fn from_json_value(v: serde_json::Value) -> Self {
        CanonicalJson::new(v).sha256_hex()
    }
}

/// Canonical JSON: objects are recursively key-sorted; arrays preserve order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalJson(pub serde_json::Value);

impl CanonicalJson {
    /// Build a canonical JSON wrapper.
    pub fn new(v: serde_json::Value) -> Self {
        CanonicalJson(canonicalize_value(v))
    }

    /// Serialize canonical value to bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        serde_json::to_vec(&self.0).expect("canonical json to_vec must not fail")
    }

    /// Hash canonical JSON bytes as SHA-256 hex.
    pub fn sha256_hex(&self) -> Sha256Hex {
        Sha256Hex::from_bytes(&self.to_vec())
    }
}

fn canonicalize_value(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for k in keys {
                let vv = map.get(&k).cloned().unwrap_or(serde_json::Value::Null);
                out.insert(k, canonicalize_value(vv));
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(canonicalize_value).collect())
        }
        other => other,
    }
}

/// Deterministic key material (SSOT).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKeyMaterial {
    /// Dataset identifier.
    pub dataset_id: String,
    /// Determinism seed.
    pub seed: u64,
    /// Numeric epsilon string.
    pub eps: String,
    /// Numeric round-step string.
    pub round_step: String,
    /// Input payload hash (sha256 hex).
    pub inputs_sha256: String,
    /// Options hash (sha256 hex).
    pub options_sha256: String,
    /// SSOT contract hash (sha256 hex).
    pub ssot_hash: String,
    /// Schema version participating in compatibility.
    pub schema_version: u32,
}

impl CacheKeyMaterial {
    /// Validate key material for minimal contract requirements.
    pub fn validate(&self) -> CacheResult<()> {
        if self.dataset_id.trim().is_empty() {
            return Err(CacheError::InvalidKey("dataset_id empty".to_string()));
        }
        if self.eps.trim().is_empty() || self.round_step.trim().is_empty() {
            return Err(CacheError::InvalidKey(
                "eps/round_step must be non-empty strings".to_string(),
            ));
        }
        for (k, v) in [
            ("inputs_sha256", &self.inputs_sha256),
            ("options_sha256", &self.options_sha256),
            ("ssot_hash", &self.ssot_hash),
        ] {
            if v.len() < 8 {
                return Err(CacheError::InvalidKey(format!("{k} too short")));
            }
            if !v.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(CacheError::InvalidKey(format!("{k} must be hex")));
            }
        }
        Ok(())
    }

    /// Render deterministic canonical JSON representation.
    pub fn to_canonical_json(&self) -> CanonicalJson {
        CanonicalJson::new(serde_json::json!({
            "dataset_id": self.dataset_id,
            "seed": self.seed,
            "eps": self.eps,
            "round_step": self.round_step,
            "inputs_sha256": self.inputs_sha256,
            "options_sha256": self.options_sha256,
            "ssot_hash": self.ssot_hash,
            "schema_version": self.schema_version
        }))
    }
}

/// Final cache key (opaque).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    /// Deterministic hash payload.
    pub hash: Sha256Hex,
}

impl CacheKey {
    /// Create a cache key from deterministic material.
    pub fn new(material: &CacheKeyMaterial) -> CacheResult<Self> {
        material.validate()?;
        let canon = material.to_canonical_json();
        Ok(CacheKey {
            hash: canon.sha256_hex(),
        })
    }

    /// Return key as hex string.
    pub fn as_str(&self) -> &str {
        &self.hash.0
    }
}
