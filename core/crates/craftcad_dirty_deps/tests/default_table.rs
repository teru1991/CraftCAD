use craftcad_dirty_deps::{default_dirty_deps_v1, ArtifactKind, ChangeKind, DirtyDepsV1, RuleV1};

#[test]
fn default_table_has_schema_v1_and_all_change_kinds() {
    let t = default_dirty_deps_v1();
    assert_eq!(t.schema_version, 1);
    assert_eq!(t.rules.len(), 8);
}

#[test]
fn canonicalize_sorts_and_dedups_invalidates() {
    let t = DirtyDepsV1 {
        schema_version: 1,
        rules: vec![RuleV1 {
            change: ChangeKind::SsotPartGeometryChanged,
            invalidates: vec![
                ArtifactKind::ViewpackV1,
                ArtifactKind::EstimateLiteV1,
                ArtifactKind::ViewpackV1,
                ArtifactKind::ProjectionLiteV1,
            ],
        }],
    }
    .canonicalize();

    assert_eq!(
        t.rules[0].invalidates,
        vec![
            ArtifactKind::EstimateLiteV1,
            ArtifactKind::ProjectionLiteV1,
            ArtifactKind::ViewpackV1,
        ]
    );
}

#[test]
fn invalidates_for_representative_change_kinds() {
    let t = default_dirty_deps_v1();

    assert_eq!(
        t.invalidates_for(ChangeKind::SsotMaterialChanged),
        vec![ArtifactKind::EstimateLiteV1, ArtifactKind::ViewpackV1]
    );

    assert_eq!(
        t.invalidates_for(ChangeKind::SsotFeatureScrewChanged),
        vec![
            ArtifactKind::FastenerBomLiteV1,
            ArtifactKind::MfgHintsLiteV1,
            ArtifactKind::ViewpackV1,
        ]
    );

    assert_eq!(
        t.invalidates_for(ChangeKind::SsotFeatureChamferChanged),
        vec![ArtifactKind::ProjectionLiteV1, ArtifactKind::ViewpackV1]
    );
}
