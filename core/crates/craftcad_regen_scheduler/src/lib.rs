use craftcad_artifact_store::{ArtifactEntryV1, ArtifactStoreV1};
use craftcad_dirty_deps::ArtifactKind;
use craftcad_dirty_engine::DirtyPlanV1;
use craftcad_estimate_lite::compute_estimate_lite;
use craftcad_mfg_hints_lite::compute_fastener_bom_with_hints_lite;
use craftcad_projection_lite::{project_to_sheet_lite, Aabb, PartBox, SheetLiteV1, ViewLite};
use craftcad_ssot::SsotV1;
use craftcad_viewpack::build_viewpack_from_ssot;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegenMode {
    Sync,
    Job,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegenJobV1 {
    pub schema_version: u32,
    pub dirty_plan: DirtyPlanV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegenError {
    pub reason_code: String,
    pub message: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn to_entry<T: Serialize>(kind: ArtifactKind, value: &T) -> Result<ArtifactEntryV1, RegenError> {
    let bytes = serde_json::to_vec(value).map_err(|e| RegenError {
        reason_code: "REGEN_SERIALIZE_FAILED".to_string(),
        message: format!("failed to serialize {kind:?}: {e}"),
    })?;

    Ok(ArtifactEntryV1 {
        kind,
        schema_version: 1,
        sha256_hex: sha256_hex(&bytes),
        bytes,
    })
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ProjectionBundleV1 {
    schema_version: u32,
    front: SheetLiteV1,
    top: SheetLiteV1,
    side: SheetLiteV1,
}

fn compute_entries_for_plan(
    ssot: &SsotV1,
    dirty_plan: &DirtyPlanV1,
) -> Result<Vec<ArtifactEntryV1>, RegenError> {
    let mut kinds: Vec<ArtifactKind> = dirty_plan.dirty.iter().map(|d| d.artifact).collect();
    kinds.sort();
    kinds.dedup();

    let mut staged = Vec::new();

    for kind in kinds {
        match kind {
            ArtifactKind::EstimateLiteV1 => {
                let estimate = compute_estimate_lite(ssot);
                staged.push(to_entry(ArtifactKind::EstimateLiteV1, &estimate)?);
            }
            ArtifactKind::ProjectionLiteV1 => {
                let boxes = part_boxes_from_ssot(ssot);
                let bundle = ProjectionBundleV1 {
                    schema_version: 1,
                    front: project_to_sheet_lite(ViewLite::Front, boxes.clone()),
                    top: project_to_sheet_lite(ViewLite::Top, boxes.clone()),
                    side: project_to_sheet_lite(ViewLite::Side, boxes),
                };
                staged.push(to_entry(ArtifactKind::ProjectionLiteV1, &bundle)?);
            }
            ArtifactKind::FastenerBomLiteV1 => {
                let bundle =
                    compute_fastener_bom_with_hints_lite(ssot).map_err(|(code, message)| {
                        RegenError {
                            reason_code: code,
                            message,
                        }
                    })?;
                staged.push(to_entry(
                    ArtifactKind::FastenerBomLiteV1,
                    &bundle.fastener_bom,
                )?);
            }
            ArtifactKind::MfgHintsLiteV1 => {
                let bundle =
                    compute_fastener_bom_with_hints_lite(ssot).map_err(|(code, message)| {
                        RegenError {
                            reason_code: code,
                            message,
                        }
                    })?;
                staged.push(to_entry(ArtifactKind::MfgHintsLiteV1, &bundle.mfg_hints)?);
            }
            ArtifactKind::ViewpackV1 => {
                let viewpack =
                    build_viewpack_from_ssot(ssot).map_err(|(code, message)| RegenError {
                        reason_code: code,
                        message,
                    })?;
                staged.push(to_entry(ArtifactKind::ViewpackV1, &viewpack)?);
            }
        }
    }

    Ok(staged)
}

pub fn compute_sync_regen(
    ssot: &SsotV1,
    dirty_plan: &DirtyPlanV1,
    store: &ArtifactStoreV1,
) -> Result<ArtifactStoreV1, RegenError> {
    let mut next = store.clone().canonicalize();
    let staged = compute_entries_for_plan(ssot, dirty_plan)?;
    for entry in staged {
        next.upsert(entry);
    }
    Ok(next.canonicalize())
}

pub fn build_regen_job(dirty_plan: &DirtyPlanV1) -> RegenJobV1 {
    RegenJobV1 {
        schema_version: 1,
        dirty_plan: dirty_plan.clone(),
    }
}

pub fn run_regen_job(
    ssot: &SsotV1,
    job: &RegenJobV1,
    store: &ArtifactStoreV1,
) -> Result<ArtifactStoreV1, RegenError> {
    compute_sync_regen(ssot, &job.dirty_plan, store)
}

pub fn run_regeneration(
    mode: RegenMode,
    ssot: &SsotV1,
    dirty_plan: &DirtyPlanV1,
    store: &ArtifactStoreV1,
) -> Result<ArtifactStoreV1, RegenError> {
    match mode {
        RegenMode::Sync => compute_sync_regen(ssot, dirty_plan, store),
        RegenMode::Job => {
            let job = build_regen_job(dirty_plan);
            run_regen_job(ssot, &job, store)
        }
    }
}
