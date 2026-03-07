use craftcad_screw_lite::eval_screw_points;
use craftcad_ssot::{
    FeatureGraphV1, FeatureNodeV1, FeatureTargetV1, FeatureTypeV1, GrainPolicyV1,
    ManufacturingOutline2dV1, MaterialCategoryV1, MaterialV1, PartV1, SsotV1,
};
use uuid::Uuid;

#[test]
fn screw_feature_fixture_includes_params_v1() {
    let material_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let part_id = Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
    let feature_id = Uuid::parse_str("00000000-0000-0000-0000-0000000000f1").unwrap();

    let params = serde_json::json!({
        "v": 1,
        "spec_name": "screw_3_5x30",
        "points": [{"x": 10.0, "y": 20.0}]
    });
    assert_eq!(params.get("v").and_then(|v| v.as_u64()), Some(1));

    let ssot = SsotV1::new(
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
            manufacturing_outline_2d: Some(ManufacturingOutline2dV1 {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 100.0,
                max_y: 100.0,
            }),
            thickness_mm: Some(18.0),
            grain_direction: None,
            labels: vec![],
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
    .canonicalize();

    let points = eval_screw_points(&ssot).expect("v1 params with points should be evaluable");
    assert_eq!(points.len(), 1);
}
