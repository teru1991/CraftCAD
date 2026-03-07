use base64::Engine;
use craftcad_estimate_lite::{compute_estimate_lite, EstimateLiteV1};
use craftcad_mfg_hints_lite::{
    compute_fastener_bom_with_hints_lite, compute_mfg_hints_lite, FastenerBomLiteV1,
    ManufacturingHintsLiteV1,
};
use craftcad_projection_lite::{project_to_sheet_lite, Aabb, PartBox, SheetLiteV1, ViewLite};
use craftcad_ssot::SsotV1;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const VIEWER_PACK_VERSION: u32 = 1;
pub const REQUIRED_ARTIFACTS: [&str; 4] = [
    "estimate_lite_v1.json",
    "projection_lite_front_v1.json",
    "fastener_bom_lite_v1.json",
    "mfg_hints_lite_v1.json",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationIssueKind {
    MissingArtifact,
    HashMismatch,
    Base64DecodeFailed,
    InvalidViewerPackVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationIssue {
    pub kind: VerificationIssueKind,
    pub artifact_name: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ViewerPackArtifactV1 {
    pub name: String,
    pub schema_version: u32,
    pub sha256_hex: String,
    pub bytes_len: usize,
    pub payload_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ViewerPackV1 {
    pub viewer_pack_version: u32,
    pub ssot_hash_hex: String,
    pub artifacts: Vec<ViewerPackArtifactV1>,
}

fn hash_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn canonical_ssot_bytes(ssot: &SsotV1) -> Vec<u8> {
    serde_json::to_vec(&ssot.clone().canonicalize())
        .expect("ssot canonical json serialize must not fail")
}

fn make_artifact<T: Serialize>(name: &str, schema_version: u32, value: &T) -> ViewerPackArtifactV1 {
    let bytes = serde_json::to_vec(value).expect("artifact json serialize must not fail");
    let sha256_hex = hash_hex(&bytes);
    let payload_base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    ViewerPackArtifactV1 {
        name: name.to_string(),
        schema_version,
        sha256_hex,
        bytes_len: bytes.len(),
        payload_base64,
    }
}

fn part_boxes_from_ssot(ssot: &SsotV1) -> Vec<PartBox> {
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
    hash_hex(&canonical_ssot_bytes(ssot))
}

pub fn build_viewpack(
    ssot: &SsotV1,
    estimate: &EstimateLiteV1,
    projection_front: &SheetLiteV1,
    fastener_bom: &FastenerBomLiteV1,
    mfg_hints: &ManufacturingHintsLiteV1,
) -> ViewerPackV1 {
    let mut artifacts = vec![
        make_artifact("estimate_lite_v1.json", estimate.schema_version, estimate),
        make_artifact(
            "projection_lite_front_v1.json",
            projection_front.schema_version,
            projection_front,
        ),
        make_artifact(
            "fastener_bom_lite_v1.json",
            fastener_bom.schema_version,
            fastener_bom,
        ),
        make_artifact(
            "mfg_hints_lite_v1.json",
            mfg_hints.schema_version,
            mfg_hints,
        ),
    ];
    artifacts.sort_by(|a, b| a.name.cmp(&b.name));

    ViewerPackV1 {
        viewer_pack_version: VIEWER_PACK_VERSION,
        ssot_hash_hex: ssot_hash_hex(ssot),
        artifacts,
    }
}

pub fn build_viewpack_from_ssot(ssot: &SsotV1) -> Result<ViewerPackV1, (String, String)> {
    let estimate = compute_estimate_lite(ssot);
    let projection_front = project_to_sheet_lite(ViewLite::Front, part_boxes_from_ssot(ssot));
    let fastener_bundle = compute_fastener_bom_with_hints_lite(ssot)?;
    let mfg_hints = compute_mfg_hints_lite(ssot)?;

    Ok(build_viewpack(
        ssot,
        &estimate,
        &projection_front,
        &fastener_bundle.fastener_bom,
        &mfg_hints,
    ))
}

pub fn verify_viewpack(viewpack: &ViewerPackV1) -> Vec<VerificationIssue> {
    let mut issues = Vec::new();

    if viewpack.viewer_pack_version != VIEWER_PACK_VERSION {
        issues.push(VerificationIssue {
            kind: VerificationIssueKind::InvalidViewerPackVersion,
            artifact_name: None,
            message: format!(
                "unsupported viewer_pack_version: {}",
                viewpack.viewer_pack_version
            ),
        });
    }

    for name in REQUIRED_ARTIFACTS {
        if !viewpack.artifacts.iter().any(|a| a.name == name) {
            issues.push(VerificationIssue {
                kind: VerificationIssueKind::MissingArtifact,
                artifact_name: Some(name.to_string()),
                message: "required artifact is missing (Not generated)".to_string(),
            });
        }
    }

    for artifact in &viewpack.artifacts {
        let decoded =
            match base64::engine::general_purpose::STANDARD.decode(&artifact.payload_base64) {
                Ok(v) => v,
                Err(e) => {
                    issues.push(VerificationIssue {
                        kind: VerificationIssueKind::Base64DecodeFailed,
                        artifact_name: Some(artifact.name.clone()),
                        message: format!("base64 decode failed: {e}"),
                    });
                    continue;
                }
            };
        if decoded.len() != artifact.bytes_len {
            issues.push(VerificationIssue {
                kind: VerificationIssueKind::HashMismatch,
                artifact_name: Some(artifact.name.clone()),
                message: format!(
                    "bytes_len mismatch: expected {}, got {}",
                    artifact.bytes_len,
                    decoded.len()
                ),
            });
            continue;
        }

        let actual = hash_hex(&decoded);
        if actual != artifact.sha256_hex {
            issues.push(VerificationIssue {
                kind: VerificationIssueKind::HashMismatch,
                artifact_name: Some(artifact.name.clone()),
                message: "artifact hash mismatch (Corrupt pack)".to_string(),
            });
        }
    }

    issues
}
