use craftcad_ssot::FeatureTypeV1;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaRef {
    pub schema_id: &'static str,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamsError {
    pub reason_code: String,
    pub message: String,
}

#[derive(Debug, Clone)]
struct Entry {
    feature_type: FeatureTypeV1,
    version: u32,
    schema_id: &'static str,
    is_latest: bool,
}

// IMPORTANT: deterministic ordering; keep sorted by feature_type then version.
const REGISTRY: &[Entry] = &[
    // ScrewFeature
    Entry {
        feature_type: FeatureTypeV1::ScrewFeature,
        version: 1,
        schema_id: "screw_feature.v1",
        is_latest: true,
    },
    // ExtrudeAdd / ExtrudeCut / Hole / Pattern / Chamfer will be added in G2-3.
    // Placeholder latest flags are not set here to avoid lying.
];

pub fn schema_id(feature_type: FeatureTypeV1, v: u32) -> Option<&'static str> {
    REGISTRY
        .iter()
        .find(|e| &e.feature_type == &feature_type && e.version == v)
        .map(|e| e.schema_id)
}

pub fn latest_version(feature_type: FeatureTypeV1) -> Option<u32> {
    REGISTRY
        .iter()
        .find(|e| &e.feature_type == &feature_type && e.is_latest)
        .map(|e| e.version)
}

pub fn validate_params(
    feature_type: FeatureTypeV1,
    params: &Value,
) -> Result<SchemaRef, ParamsError> {
    let v = params
        .get("v")
        .and_then(|x| x.as_u64())
        .ok_or_else(|| ParamsError {
            reason_code: "FEATURE_PARAMS_SCHEMA_MISMATCH".to_string(),
            message: "params missing required numeric field: v".to_string(),
        })? as u32;

    let sid = schema_id(feature_type.clone(), v).ok_or_else(|| ParamsError {
        reason_code: "FEATURE_PARAMS_SCHEMA_MISMATCH".to_string(),
        message: format!(
            "unsupported params schema for feature={:?} v={}",
            feature_type, v
        ),
    })?;

    Ok(SchemaRef {
        schema_id: sid,
        version: v,
    })
}
