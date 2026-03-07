use craftcad_dirty_deps::ArtifactKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactEntryV1 {
    pub kind: ArtifactKind,
    pub schema_version: u32,
    pub sha256_hex: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactStoreV1 {
    pub schema_version: u32, // = 1
    pub entries: Vec<ArtifactEntryV1>,
}

impl Default for ArtifactStoreV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            entries: Vec::new(),
        }
    }
}

impl ArtifactStoreV1 {
    pub fn canonicalize(mut self) -> Self {
        let mut by_kind: BTreeMap<ArtifactKind, ArtifactEntryV1> = BTreeMap::new();
        for entry in self.entries.drain(..) {
            by_kind.insert(entry.kind, entry);
        }
        self.entries = by_kind.into_values().collect();
        self
    }

    pub fn upsert(&mut self, entry: ArtifactEntryV1) {
        self.entries.push(entry);
        let canonical = std::mem::take(self).canonicalize();
        *self = canonical;
    }
}
