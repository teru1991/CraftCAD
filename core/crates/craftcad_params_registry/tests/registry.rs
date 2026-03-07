use craftcad_params_registry::{latest_version, schema_id, validate_params};
use craftcad_ssot::FeatureTypeV1;

#[test]
fn screw_v1_schema_resolves_and_latest_is_one() {
    assert_eq!(
        schema_id(FeatureTypeV1::ScrewFeature, 1),
        Some("screw_feature.v1")
    );
    assert_eq!(latest_version(FeatureTypeV1::ScrewFeature), Some(1));
}

#[test]
fn validate_fails_when_v_missing() {
    let err = validate_params(
        FeatureTypeV1::ScrewFeature,
        &serde_json::json!({"spec_name":"M4"}),
    )
    .expect_err("missing v must fail");
    assert_eq!(err.reason_code, "FEATURE_PARAMS_SCHEMA_MISMATCH");
}

#[test]
fn validate_fails_on_unknown_version() {
    let err = validate_params(FeatureTypeV1::ScrewFeature, &serde_json::json!({"v":2}))
        .expect_err("unknown version must fail");
    assert_eq!(err.reason_code, "FEATURE_PARAMS_SCHEMA_MISMATCH");
}

#[test]
fn determinism_registry_first_entry_is_screw_v1() {
    let r = validate_params(FeatureTypeV1::ScrewFeature, &serde_json::json!({"v":1}))
        .expect("known schema should validate");
    assert_eq!(r.schema_id, "screw_feature.v1");
    assert_eq!(r.version, 1);
}
