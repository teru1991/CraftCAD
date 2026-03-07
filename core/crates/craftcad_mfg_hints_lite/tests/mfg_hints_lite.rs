use craftcad_mfg_hints_lite::{
    compute_fastener_bom_with_hints_lite, compute_mfg_hints_lite, hints_hash_hex,
};
use craftcad_ssot::{
    FeatureGraphV1, FeatureNodeV1, FeatureTargetV1, FeatureTypeV1, GrainPolicyV1,
    MaterialCategoryV1, MaterialV1, PartV1, SsotV1,
};
use diycad_project::{create_empty_project, load, save, DiycadProject};
use tempfile::tempdir;
use uuid::Uuid;

fn sample_ssot(features: Vec<FeatureNodeV1>) -> SsotV1 {
    let material_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let part_id = Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
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
            name: "p1".to_string(),
            material_id,
            quantity: 1,
            manufacturing_outline_2d: None,
            thickness_mm: Some(18.0),
            grain_direction: None,
            labels: vec![],
            feature_ids: features.iter().map(|f| f.feature_id).collect(),
        }],
        FeatureGraphV1 { features },
    )
    .canonicalize()
}

fn screw_feature(feature_id: Uuid, part_id: Uuid, params: serde_json::Value) -> FeatureNodeV1 {
    FeatureNodeV1 {
        feature_id,
        feature_type: FeatureTypeV1::ScrewFeature,
        params,
        targets: vec![FeatureTargetV1 { part_id }],
    }
}

#[test]
fn pilot_hole_value_is_used_in_note() {
    let part_id = Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
    let fid = Uuid::parse_str("00000000-0000-0000-0000-0000000000f1").unwrap();
    let ssot = sample_ssot(vec![screw_feature(
        fid,
        part_id,
        serde_json::json!({"v":1,"spec_name":"screw_3_5x30","pilot_hole_mm":2.6}),
    )]);
    let hints = compute_mfg_hints_lite(&ssot).unwrap();
    assert_eq!(hints.items[0].pilot_hole_mm, 2.6);
    assert!(hints.items[0].note_text.contains("pilot φ2.6mm"));
}

#[test]
fn default_pilot_hole_is_3mm() {
    let part_id = Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
    let fid = Uuid::parse_str("00000000-0000-0000-0000-0000000000f2").unwrap();
    let ssot = sample_ssot(vec![screw_feature(
        fid,
        part_id,
        serde_json::json!({"v":1,"spec_name":"screw_3_5x30"}),
    )]);
    let hints = compute_mfg_hints_lite(&ssot).unwrap();
    assert_eq!(hints.items[0].pilot_hole_mm, 3.0);
}

#[test]
fn countersink_text_and_depth_added() {
    let part_id = Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
    let fid = Uuid::parse_str("00000000-0000-0000-0000-0000000000f3").unwrap();
    let ssot = sample_ssot(vec![screw_feature(
        fid,
        part_id,
        serde_json::json!({"v":1,"spec_name":"screw_3_5x30","countersink":true,"countersink_depth_mm":1.2345}),
    )]);
    let hints = compute_mfg_hints_lite(&ssot).unwrap();
    assert!(hints.items[0].note_text.contains("countersink"));
    assert!(hints.items[0].note_text.contains("depth 1.235mm"));
}

#[test]
fn determinism_hash_independent_of_feature_order() {
    let part_id = Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
    let f1 = screw_feature(
        Uuid::parse_str("00000000-0000-0000-0000-0000000000f1").unwrap(),
        part_id,
        serde_json::json!({"v":1,"spec_name":"s1","pilot_hole_mm":2.5}),
    );
    let f2 = screw_feature(
        Uuid::parse_str("00000000-0000-0000-0000-0000000000f2").unwrap(),
        part_id,
        serde_json::json!({"v":1,"spec_name":"s2","pilot_hole_mm":3.0}),
    );
    let h1 = hints_hash_hex(
        &compute_mfg_hints_lite(&sample_ssot(vec![f1.clone(), f2.clone()])).unwrap(),
    );
    let h2 = hints_hash_hex(&compute_mfg_hints_lite(&sample_ssot(vec![f2, f1])).unwrap());
    assert_eq!(h1, h2);
}

#[test]
fn invalid_params_return_reason_code() {
    let part_id = Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
    let fid = Uuid::parse_str("00000000-0000-0000-0000-0000000000ff").unwrap();
    let ssot = sample_ssot(vec![screw_feature(
        fid,
        part_id,
        serde_json::json!({"v":2,"spec_name":"s1"}),
    )]);
    let err = compute_mfg_hints_lite(&ssot).unwrap_err();
    assert_eq!(err.0, "HINTS_UNSUPPORTED_FEATURE");
}

#[test]
fn integration_roundtrip_preserves_recomputed_hints_hash() {
    let part_id = Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
    let fid = Uuid::parse_str("00000000-0000-0000-0000-0000000000f1").unwrap();
    let ssot = sample_ssot(vec![screw_feature(
        fid,
        part_id,
        serde_json::json!({"v":1,"spec_name":"screw_3_5x30","pilot_hole_mm":2.4,"countersink":true}),
    )]);

    let before = compute_mfg_hints_lite(&ssot).unwrap();
    let before_hash = hints_hash_hex(&before);
    let bundle = compute_fastener_bom_with_hints_lite(&ssot).unwrap();
    assert_eq!(bundle.mfg_hints.items.len(), 1);
    assert_eq!(bundle.fastener_bom.items.len(), 1);

    let mut project: DiycadProject = create_empty_project("0.1.0", "mm", "2026-03-01T00:00:00Z");
    project.ssot_v1 = Some(ssot.clone());

    let dir = tempdir().unwrap();
    let path = dir.path().join("hints_roundtrip.diycad");
    save(&path, &project).unwrap();
    let loaded = load(&path).unwrap();
    let after = compute_mfg_hints_lite(loaded.ssot_v1.as_ref().unwrap()).unwrap();
    let after_hash = hints_hash_hex(&after);
    assert_eq!(before_hash, after_hash);
}
