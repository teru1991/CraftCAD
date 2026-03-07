use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    EstimateLiteV1,
    ProjectionLiteV1,
    FastenerBomLiteV1,
    MfgHintsLiteV1,
    ViewpackV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    SsotMaterialChanged,
    SsotPartGeometryChanged,
    SsotPartQuantityChanged,
    SsotFeatureScrewChanged,
    SsotFeatureHoleChanged,
    SsotFeaturePatternChanged,
    SsotFeatureExtrudeChanged,
    SsotFeatureChamferChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuleV1 {
    pub change: ChangeKind,
    pub invalidates: Vec<ArtifactKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirtyDepsV1 {
    pub schema_version: u32, // = 1
    pub rules: Vec<RuleV1>,
}

impl DirtyDepsV1 {
    pub fn canonicalize(mut self) -> Self {
        for r in self.rules.iter_mut() {
            r.invalidates.sort();
            r.invalidates.dedup();
        }
        self.rules.sort_by_key(|r| r.change);
        self
    }

    pub fn invalidates_for(&self, change: ChangeKind) -> Vec<ArtifactKind> {
        self.rules
            .iter()
            .find(|r| r.change == change)
            .map(|r| r.invalidates.clone())
            .unwrap_or_default()
    }
}

pub fn default_dirty_deps_v1() -> DirtyDepsV1 {
    DirtyDepsV1 {
        schema_version: 1,
        rules: vec![
            RuleV1 {
                change: ChangeKind::SsotMaterialChanged,
                invalidates: vec![ArtifactKind::EstimateLiteV1, ArtifactKind::ViewpackV1],
            },
            RuleV1 {
                change: ChangeKind::SsotPartGeometryChanged,
                invalidates: vec![
                    ArtifactKind::EstimateLiteV1,
                    ArtifactKind::ProjectionLiteV1,
                    ArtifactKind::ViewpackV1,
                ],
            },
            RuleV1 {
                change: ChangeKind::SsotPartQuantityChanged,
                invalidates: vec![
                    ArtifactKind::EstimateLiteV1,
                    ArtifactKind::FastenerBomLiteV1,
                    ArtifactKind::ViewpackV1,
                ],
            },
            RuleV1 {
                change: ChangeKind::SsotFeatureScrewChanged,
                invalidates: vec![
                    ArtifactKind::FastenerBomLiteV1,
                    ArtifactKind::MfgHintsLiteV1,
                    ArtifactKind::ViewpackV1,
                ],
            },
            RuleV1 {
                change: ChangeKind::SsotFeatureHoleChanged,
                invalidates: vec![ArtifactKind::ProjectionLiteV1, ArtifactKind::ViewpackV1],
            },
            RuleV1 {
                change: ChangeKind::SsotFeaturePatternChanged,
                invalidates: vec![
                    ArtifactKind::ProjectionLiteV1,
                    ArtifactKind::FastenerBomLiteV1,
                    ArtifactKind::ViewpackV1,
                ],
            },
            RuleV1 {
                change: ChangeKind::SsotFeatureExtrudeChanged,
                invalidates: vec![
                    ArtifactKind::ProjectionLiteV1,
                    ArtifactKind::EstimateLiteV1,
                    ArtifactKind::ViewpackV1,
                ],
            },
            RuleV1 {
                change: ChangeKind::SsotFeatureChamferChanged,
                invalidates: vec![ArtifactKind::ProjectionLiteV1, ArtifactKind::ViewpackV1],
            },
        ],
    }
    .canonicalize()
}
