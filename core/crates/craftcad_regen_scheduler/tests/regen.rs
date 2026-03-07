use craftcad_artifact_store::ArtifactStoreV1;
use craftcad_dirty_deps::ArtifactKind;
use craftcad_dirty_engine::{DirtyArtifactV1, DirtyPlanV1};
use craftcad_regen_scheduler::{compute_sync_regen, run_regeneration, RegenMode};
use craftcad_ssot::{
    deterministic_uuid, FeatureGraphV1, FeatureNodeV1, FeatureTargetV1, FeatureTypeV1,
    GrainPolicyV1, ManufacturingOutline2dV1, MaterialCategoryV1, MaterialV1, PartV1, SsotV1,
};

fn sample_ssot(valid_screw: bool) -> SsotV1 {
    let material_id = deterministic_uuid("material", "regen");
    let part_id = deterministic_uuid("part", "regen");
    let feature_id = deterministic_uuid("feature", "regen");

    let params = if valid_screw {
        serde_json::json!({"v":1,"spec_name":"M4","pilot_hole_mm":3.2,"countersink":false})
    } else {
        serde_json::json!({"v":2,"spec_name":"bad"})
    };

    SsotV1::new(
        vec![MaterialV1 {
            material_id,
            category: MaterialCategoryV1::Unspecified,
            name: "mat".to_string(),
            thickness_mm: Some(18.0),
            grain_policy: GrainPolicyV1::None,
            kerf_mm: 2.0,
            margin_mm: 5.0,
            estimate_loss_factor: None,
        }],
        vec![PartV1 {
            part_id,
            name: "p1".to_string(),
            material_id,
            quantity: 1,
            manufacturing_outline_2d: Some(ManufacturingOutline2dV1 {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 100.0,
                max_y: 50.0,
            }),
            thickness_mm: Some(18.0),
            grain_direction: None,
            labels: Vec::new(),
            feature_ids: vec![feature_id],
        }],
        FeatureGraphV1 {
            features: vec![FeatureNodeV1 {
                feature_id,
                feature_type: FeatureTypeV1::ScrewFeature,
                params,
                targets: vec![FeatureTargetV1 { part_id }],
            }],
        },
    )
    .canonicalize()
}

fn dirty_plan() -> DirtyPlanV1 {
    DirtyPlanV1 {
        schema_version: 1,
        dirty: vec![
            DirtyArtifactV1 {
                artifact: ArtifactKind::EstimateLiteV1,
                reasons: vec![],
            },
            DirtyArtifactV1 {
                artifact: ArtifactKind::FastenerBomLiteV1,
                reasons: vec![],
            },
            DirtyArtifactV1 {
                artifact: ArtifactKind::ViewpackV1,
                reasons: vec![],
            },
        ],
    }
}

#[test]
fn sync_regen_produces_expected_entries() {
    let out = compute_sync_regen(
        &sample_ssot(true),
        &dirty_plan(),
        &ArtifactStoreV1::default(),
    )
    .expect("sync regen should succeed");

    let kinds: Vec<_> = out.entries.iter().map(|e| e.kind).collect();
    assert!(kinds.contains(&ArtifactKind::EstimateLiteV1));
    assert!(kinds.contains(&ArtifactKind::FastenerBomLiteV1));
    assert!(kinds.contains(&ArtifactKind::ViewpackV1));
    assert!(out.entries.iter().all(|e| e.sha256_hex.len() == 64));
}

#[test]
fn empty_dirty_plan_keeps_store_unchanged() {
    let store = ArtifactStoreV1::default();
    let plan = DirtyPlanV1 {
        schema_version: 1,
        dirty: vec![],
    };
    let out = compute_sync_regen(&sample_ssot(true), &plan, &store).expect("regen should pass");
    assert_eq!(out, store);
}

#[test]
fn failure_is_all_or_nothing() {
    let initial = ArtifactStoreV1::default();
    let err = compute_sync_regen(&sample_ssot(false), &dirty_plan(), &initial)
        .expect_err("regen must fail on invalid screw params");
    assert_eq!(err.reason_code, "HINTS_UNSUPPORTED_FEATURE");
    assert_eq!(initial.entries.len(), 0);
}

#[test]
fn job_mode_matches_sync_mode() {
    let ssot = sample_ssot(true);
    let plan = dirty_plan();
    let sync = run_regeneration(RegenMode::Sync, &ssot, &plan, &ArtifactStoreV1::default())
        .expect("sync mode should succeed");
    let job = run_regeneration(RegenMode::Job, &ssot, &plan, &ArtifactStoreV1::default())
        .expect("job mode should succeed");
    assert_eq!(sync, job);
}
