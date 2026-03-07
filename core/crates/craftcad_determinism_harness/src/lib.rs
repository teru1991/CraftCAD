use craftcad_estimate_lite::{compute_estimate_lite, estimate_hash_hex};
use craftcad_mfg_hints_lite::{compute_fastener_bom_with_hints_lite, fastener_bom_hash_hex};
use craftcad_projection_lite::{project_to_sheet_lite, sheet_hash_hex, Aabb, PartBox, ViewLite};
use craftcad_ssot::{
    deterministic_uuid, FeatureGraphV1, FeatureNodeV1, FeatureTargetV1, FeatureTypeV1,
    GrainPolicyV1, ManufacturingOutline2dV1, MaterialCategoryV1, MaterialV1, PartV1, SsotV1,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectionHashes {
    pub front: String,
    pub top: String,
    pub side: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeterminismSummary {
    pub ok: bool,
    pub projection: ProjectionHashes,
    pub estimate: String,
    pub fastener_bom: String,
    pub input_ssot_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunHashes {
    pub projection: ProjectionHashes,
    pub estimate: String,
    pub fastener_bom: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckResult {
    pub summary: DeterminismSummary,
    pub runs: Vec<RunHashes>,
}

fn to_projection_part_boxes(ssot: &SsotV1) -> Vec<PartBox> {
    let mut parts = ssot.parts.clone();
    parts.sort_by_key(|p| p.part_id);
    parts
        .into_iter()
        .map(|part| {
            let aabb = match part.manufacturing_outline_2d {
                Some(outline) => Aabb {
                    min_x: outline.min_x.min(outline.max_x),
                    min_y: outline.min_y.min(outline.max_y),
                    min_z: 0.0,
                    max_x: outline.max_x.max(outline.min_x),
                    max_y: outline.max_y.max(outline.min_y),
                    max_z: part.thickness_mm.unwrap_or(0.0).max(0.0),
                },
                None => Aabb {
                    min_x: 0.0,
                    min_y: 0.0,
                    min_z: 0.0,
                    max_x: 100.0,
                    max_y: 100.0,
                    max_z: part.thickness_mm.unwrap_or(0.0).max(0.0),
                },
            };
            PartBox {
                part_id: part.part_id,
                aabb,
            }
        })
        .collect()
}

pub fn ssot_hash_hex(ssot: &SsotV1) -> String {
    let canonical = ssot.clone().canonicalize();
    let bytes = serde_json::to_vec(&canonical).expect("ssot canonical serialization must not fail");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn compute_once(ssot: &SsotV1) -> Result<RunHashes, String> {
    let boxes = to_projection_part_boxes(ssot);
    let front = sheet_hash_hex(&project_to_sheet_lite(ViewLite::Front, boxes.clone()));
    let top = sheet_hash_hex(&project_to_sheet_lite(ViewLite::Top, boxes.clone()));
    let side = sheet_hash_hex(&project_to_sheet_lite(ViewLite::Side, boxes));

    let estimate = estimate_hash_hex(&compute_estimate_lite(ssot));

    let fastener_bom = {
        let bundle = compute_fastener_bom_with_hints_lite(ssot)
            .map_err(|(code, msg)| format!("{code}: {msg}"))?;
        fastener_bom_hash_hex(&bundle.fastener_bom)
    };

    Ok(RunHashes {
        projection: ProjectionHashes { front, top, side },
        estimate,
        fastener_bom,
    })
}

pub fn run_check(ssot: &SsotV1, n_runs: usize) -> Result<CheckResult, String> {
    let n_runs = n_runs.max(1);
    let mut runs = Vec::with_capacity(n_runs);
    for _ in 0..n_runs {
        runs.push(compute_once(ssot)?);
    }

    let first = runs[0].clone();
    let ok = runs.iter().all(|r| r == &first);

    Ok(CheckResult {
        summary: DeterminismSummary {
            ok,
            projection: first.projection,
            estimate: first.estimate,
            fastener_bom: first.fastener_bom,
            input_ssot_hash: ssot_hash_hex(ssot),
        },
        runs,
    })
}

pub fn fixture_ssot() -> SsotV1 {
    let material_id = deterministic_uuid("material", "determinism-fixture");
    let part_a = deterministic_uuid("part", "determinism-fixture:a");
    let part_b = deterministic_uuid("part", "determinism-fixture:b");
    let feature_a = deterministic_uuid("feature", "determinism-fixture:fa");
    let feature_b = deterministic_uuid("feature", "determinism-fixture:fb");

    SsotV1::new(
        vec![MaterialV1 {
            material_id,
            category: MaterialCategoryV1::Wood,
            name: "plywood".to_string(),
            thickness_mm: Some(18.0),
            grain_policy: GrainPolicyV1::None,
            kerf_mm: 2.0,
            margin_mm: 5.0,
            estimate_loss_factor: Some(1.1),
        }],
        vec![
            PartV1 {
                part_id: part_a,
                name: "a".to_string(),
                material_id,
                quantity: 2,
                manufacturing_outline_2d: Some(ManufacturingOutline2dV1 {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 120.0,
                    max_y: 80.0,
                }),
                thickness_mm: Some(18.0),
                grain_direction: None,
                labels: vec![],
                feature_ids: vec![feature_a],
            },
            PartV1 {
                part_id: part_b,
                name: "b".to_string(),
                material_id,
                quantity: 1,
                manufacturing_outline_2d: Some(ManufacturingOutline2dV1 {
                    min_x: 0.0,
                    min_y: 0.0,
                    max_x: 90.0,
                    max_y: 40.0,
                }),
                thickness_mm: Some(18.0),
                grain_direction: None,
                labels: vec![],
                feature_ids: vec![feature_b],
            },
        ],
        FeatureGraphV1 {
            features: vec![
                FeatureNodeV1 {
                    feature_id: feature_a,
                    feature_type: FeatureTypeV1::ScrewFeature,
                    params: serde_json::json!({
                        "v": 1,
                        "spec_name": "screw_3_5x30",
                        "pilot_hole_mm": 2.5,
                        "points": [{"x": 20.0, "y": 20.0}]
                    }),
                    targets: vec![FeatureTargetV1 { part_id: part_a }],
                },
                FeatureNodeV1 {
                    feature_id: feature_b,
                    feature_type: FeatureTypeV1::ScrewFeature,
                    params: serde_json::json!({
                        "v": 1,
                        "spec_name": "screw_4x40",
                        "countersink": true,
                        "points": [{"x": 10.0, "y": 10.0}]
                    }),
                    targets: vec![FeatureTargetV1 { part_id: part_b }],
                },
            ],
        },
    )
    .canonicalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n_runs_are_identical() {
        let ssot = fixture_ssot();
        let result = run_check(&ssot, 3).unwrap();
        assert!(result.summary.ok);
        assert_eq!(result.runs.len(), 3);
        assert_eq!(result.runs[0], result.runs[1]);
        assert_eq!(result.runs[1], result.runs[2]);
    }

    #[test]
    fn canonicalization_keeps_hash_stable_under_part_order_perturbation() {
        let mut a = fixture_ssot();
        let mut b = fixture_ssot();
        b.parts.reverse();

        let ra = run_check(&a, 1).unwrap();
        let rb = run_check(&b, 1).unwrap();
        assert_eq!(ra.summary.input_ssot_hash, rb.summary.input_ssot_hash);
        assert_eq!(ra.summary.projection, rb.summary.projection);
        assert_eq!(ra.summary.estimate, rb.summary.estimate);
        assert_eq!(ra.summary.fastener_bom, rb.summary.fastener_bom);

        // keep variable mutable usage explicit to avoid accidental lint changes
        a.parts.sort_by_key(|p| p.part_id);
        assert_eq!(a.parts.len(), b.parts.len());
    }
}
