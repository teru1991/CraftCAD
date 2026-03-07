use craftcad_ssot::{
    FeatureGraphV1, FeatureNodeV1, FeatureTargetV1, FeatureTypeV1, GrainPolicyV1,
    ManufacturingOutline2dV1, MaterialCategoryV1, MaterialV1, PartV1, SsotV1,
};
use craftcad_viewpack::{
    build_viewpack_from_ssot, ssot_hash_hex, verify_viewpack, VerificationIssueKind,
};
use uuid::Uuid;

fn sample_ssot(feature_order_flip: bool) -> SsotV1 {
    let material_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let part_id = Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
    let mut features = vec![
        FeatureNodeV1 {
            feature_id: Uuid::parse_str("00000000-0000-0000-0000-0000000000f1").unwrap(),
            feature_type: FeatureTypeV1::ScrewFeature,
            params: serde_json::json!({
                "v": 1,
                "spec_name": "screw_3_5x30",
                "pilot_hole_mm": 2.5,
                "countersink": true,
                "countersink_depth_mm": 1.2,
                "points": [{"x": 5.0, "y": 50.0}]
            }),
            targets: vec![FeatureTargetV1 { part_id }],
        },
        FeatureNodeV1 {
            feature_id: Uuid::parse_str("00000000-0000-0000-0000-0000000000f2").unwrap(),
            feature_type: FeatureTypeV1::ScrewFeature,
            params: serde_json::json!({
                "v": 1,
                "spec_name": "screw_4x40",
                "points": [{"x": 20.0, "y": 50.0}]
            }),
            targets: vec![FeatureTargetV1 { part_id }],
        },
    ];
    if feature_order_flip {
        features.reverse();
    }

    SsotV1::new(
        vec![MaterialV1 {
            material_id,
            category: MaterialCategoryV1::Wood,
            name: "plywood".to_string(),
            thickness_mm: Some(18.0),
            grain_policy: GrainPolicyV1::None,
            kerf_mm: 2.0,
            margin_mm: 5.0,
            estimate_loss_factor: None,
        }],
        vec![PartV1 {
            part_id,
            name: "part".to_string(),
            material_id,
            quantity: 1,
            manufacturing_outline_2d: Some(ManufacturingOutline2dV1 {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 120.0,
                max_y: 80.0,
            }),
            thickness_mm: Some(18.0),
            grain_direction: None,
            labels: vec![],
            feature_ids: features.iter().map(|f| f.feature_id).collect(),
        }],
        FeatureGraphV1 { features },
    )
}

#[test]
fn manifest_order_is_deterministic() {
    let vp = build_viewpack_from_ssot(&sample_ssot(false)).unwrap();
    let names: Vec<_> = vp.artifacts.iter().map(|a| a.name.clone()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn verify_catches_hash_mismatch() {
    let mut vp = build_viewpack_from_ssot(&sample_ssot(false)).unwrap();
    vp.artifacts[0].payload_base64 = "e30=".to_string();
    let issues = verify_viewpack(&vp);
    assert!(issues
        .iter()
        .any(|i| i.kind == VerificationIssueKind::HashMismatch));
}

#[test]
fn verify_detects_missing_artifact_as_not_generated() {
    let mut vp = build_viewpack_from_ssot(&sample_ssot(false)).unwrap();
    vp.artifacts.retain(|a| a.name != "mfg_hints_lite_v1.json");
    let issues = verify_viewpack(&vp);
    assert!(issues.iter().any(|i| {
        i.kind == VerificationIssueKind::MissingArtifact
            && i.artifact_name.as_deref() == Some("mfg_hints_lite_v1.json")
    }));
}

#[test]
fn ssot_hash_uses_canonical_ssot() {
    let ssot = sample_ssot(false);
    let h = ssot_hash_hex(&ssot);
    let canonical_bytes = serde_json::to_vec(&ssot.clone().canonicalize()).unwrap();
    let expected = {
        use sha2::{Digest, Sha256};
        hex::encode(Sha256::digest(canonical_bytes))
    };
    assert_eq!(h, expected);
}
