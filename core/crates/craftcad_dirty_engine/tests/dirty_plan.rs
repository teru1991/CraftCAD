use craftcad_dirty_deps::{default_dirty_deps_v1, ArtifactKind, ChangeKind};
use craftcad_dirty_engine::compute_dirty_plan;

#[test]
fn empty_changes_produces_empty_dirty() {
    let deps = default_dirty_deps_v1();
    let plan = compute_dirty_plan(&deps, &[]);
    assert_eq!(plan.schema_version, 1);
    assert!(plan.dirty.is_empty());
}

#[test]
fn duplicate_changes_produce_same_plan() {
    let deps = default_dirty_deps_v1();
    let a = compute_dirty_plan(
        &deps,
        &[
            ChangeKind::SsotFeatureScrewChanged,
            ChangeKind::SsotFeatureScrewChanged,
        ],
    );
    let b = compute_dirty_plan(&deps, &[ChangeKind::SsotFeatureScrewChanged]);
    assert_eq!(a, b);
}

#[test]
fn ordering_is_stable_by_artifact_kind() {
    let deps = default_dirty_deps_v1();
    let plan = compute_dirty_plan(
        &deps,
        &[
            ChangeKind::SsotFeaturePatternChanged,
            ChangeKind::SsotPartQuantityChanged,
        ],
    );

    let artifacts: Vec<ArtifactKind> = plan.dirty.iter().map(|d| d.artifact).collect();
    let mut sorted = artifacts.clone();
    sorted.sort();
    assert_eq!(artifacts, sorted);
}

#[test]
fn reasons_are_unique_and_sorted() {
    let deps = default_dirty_deps_v1();
    let plan = compute_dirty_plan(
        &deps,
        &[
            ChangeKind::SsotFeaturePatternChanged,
            ChangeKind::SsotFeaturePatternChanged,
            ChangeKind::SsotPartQuantityChanged,
        ],
    );

    let fastener = plan
        .dirty
        .iter()
        .find(|d| d.artifact == ArtifactKind::FastenerBomLiteV1)
        .expect("fastener artifact should be dirty");

    assert_eq!(
        fastener.reasons,
        vec![
            ChangeKind::SsotPartQuantityChanged,
            ChangeKind::SsotFeaturePatternChanged,
        ]
    );
}

#[test]
fn representative_mapping_for_screw_change() {
    let deps = default_dirty_deps_v1();
    let plan = compute_dirty_plan(&deps, &[ChangeKind::SsotFeatureScrewChanged]);

    let artifacts: Vec<ArtifactKind> = plan.dirty.iter().map(|d| d.artifact).collect();
    assert_eq!(
        artifacts,
        vec![
            ArtifactKind::FastenerBomLiteV1,
            ArtifactKind::MfgHintsLiteV1,
            ArtifactKind::ViewpackV1,
        ]
    );
}
