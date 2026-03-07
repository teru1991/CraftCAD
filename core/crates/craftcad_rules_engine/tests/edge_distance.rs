use craftcad_rules_engine::{preflight_rules, run_rules_edge_distance_with_points, RuleConfig};
use craftcad_screw_lite::ScrewPoint;
use craftcad_ssot::{
    FeatureGraphV1, FeatureNodeV1, FeatureTargetV1, FeatureTypeV1, GrainPolicyV1,
    ManufacturingOutline2dV1, MaterialCategoryV1, MaterialV1, PartV1, SsotV1,
};
use uuid::Uuid;

fn base_ssot(with_outline: bool, feature_id: Uuid, part_id: Uuid) -> SsotV1 {
    let material_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
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
            manufacturing_outline_2d: with_outline.then_some(ManufacturingOutline2dV1 {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 100.0,
                max_y: 100.0,
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
                params: serde_json::json!({
                    "v": 1,
                    "points": [
                        {"x": 5.0, "y": 50.0}
                    ]
                }),
                targets: vec![FeatureTargetV1 { part_id }],
            }],
        },
    )
    .canonicalize()
}

#[test]
fn edge_distance_violation_is_fatal() {
    let part_id = Uuid::new_v4();
    let feature_id = Uuid::new_v4();
    let ssot = base_ssot(true, feature_id, part_id);
    let report = run_rules_edge_distance_with_points(
        &ssot,
        RuleConfig {
            min_edge_distance_mm: 10.0,
        },
        &[ScrewPoint {
            part_id,
            feature_id,
            x: 5.0,
            y: 50.0,
        }],
    );
    assert!(report.has_fatal());
    assert_eq!(
        report.findings[0].reason_code,
        "RULE_EDGE_DISTANCE_VIOLATION"
    );
}

#[test]
fn center_point_has_no_findings() {
    let part_id = Uuid::new_v4();
    let feature_id = Uuid::new_v4();
    let ssot = base_ssot(true, feature_id, part_id);
    let report = run_rules_edge_distance_with_points(
        &ssot,
        RuleConfig::default(),
        &[ScrewPoint {
            part_id,
            feature_id,
            x: 50.0,
            y: 50.0,
        }],
    );
    assert!(report.findings.is_empty());
}

#[test]
fn missing_outline_is_warn_not_fatal() {
    let part_id = Uuid::new_v4();
    let feature_id = Uuid::new_v4();
    let ssot = base_ssot(false, feature_id, part_id);
    let report = run_rules_edge_distance_with_points(
        &ssot,
        RuleConfig::default(),
        &[ScrewPoint {
            part_id,
            feature_id,
            x: 5.0,
            y: 50.0,
        }],
    );
    assert!(!report.has_fatal());
    assert_eq!(report.findings[0].reason_code, "RULE_INPUT_MISSING");
}

#[test]
fn deterministic_json_ordering() {
    let part_id = Uuid::new_v4();
    let feature_id = Uuid::new_v4();
    let ssot = base_ssot(false, feature_id, part_id);
    let points = vec![
        ScrewPoint {
            part_id,
            feature_id,
            x: 5.0,
            y: 50.0,
        },
        ScrewPoint {
            part_id,
            feature_id,
            x: 6.0,
            y: 50.0,
        },
    ];
    let j1 = serde_json::to_string(&run_rules_edge_distance_with_points(
        &ssot,
        RuleConfig::default(),
        &points,
    ))
    .unwrap();
    let j2 = serde_json::to_string(&run_rules_edge_distance_with_points(
        &ssot,
        RuleConfig::default(),
        &points,
    ))
    .unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn preflight_blocks_on_fatal() {
    let part_id = Uuid::new_v4();
    let feature_id = Uuid::new_v4();
    let ssot = base_ssot(true, feature_id, part_id);
    let err = preflight_rules(&ssot, RuleConfig::default()).unwrap_err();
    assert_eq!(err.reason_code, "RULE_EDGE_DISTANCE_VIOLATION");
}
