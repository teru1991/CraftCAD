use craftcad_dirty_deps::{ArtifactKind, ChangeKind, DirtyDepsV1};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirtyArtifactV1 {
    pub artifact: ArtifactKind,
    pub reasons: Vec<ChangeKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirtyPlanV1 {
    pub schema_version: u32, // = 1
    pub dirty: Vec<DirtyArtifactV1>,
}

pub fn compute_dirty_plan(deps: &DirtyDepsV1, changes: &[ChangeKind]) -> DirtyPlanV1 {
    // Normalize changes (deterministic)
    let mut norm: BTreeSet<ChangeKind> = BTreeSet::new();
    for c in changes {
        norm.insert(*c);
    }

    // artifact -> reasons
    let mut map: BTreeMap<ArtifactKind, BTreeSet<ChangeKind>> = BTreeMap::new();
    for c in norm.iter() {
        for a in deps.invalidates_for(*c).into_iter() {
            map.entry(a).or_default().insert(*c);
        }
    }

    let mut dirty: Vec<DirtyArtifactV1> = map
        .into_iter()
        .map(|(artifact, reasons)| DirtyArtifactV1 {
            artifact,
            reasons: reasons.into_iter().collect(),
        })
        .collect();

    // Deterministic ordering by ArtifactKind
    dirty.sort_by_key(|d| d.artifact);

    DirtyPlanV1 {
        schema_version: 1,
        dirty,
    }
}
