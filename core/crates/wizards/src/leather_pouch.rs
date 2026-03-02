use crate::parts::model::*;
use crate::parts::{normalize::normalize, validate::validate_parts};
use crate::reasons::{WizardReason, WizardReasonCode};
use crate::types::EvaluatedOp;

fn num(v: &serde_json::Value) -> Result<f64, WizardReason> {
    v.as_f64()
        .ok_or_else(|| WizardReason::new(WizardReasonCode::WizardDslInvalid, "expected number"))
}

pub fn build_from_evaluated(
    tpl_id: &str,
    seed: u64,
    ops: &[EvaluatedOp],
    material_id: &str,
    process_id: &str,
    kerf_mm: f64,
    margin_mm: f64,
    grain: &str,
    seam_allowance_mm: f64,
    hole_pitch_mm: f64,
) -> Result<PartsDraft, WizardReason> {
    let mut parts: Vec<PartDraft> = vec![];
    let mut annotations: Vec<AnnotationDraft> = vec![];
    let mut nest: Option<NestJobDraft> = None;

    for op in ops {
        match op.op.as_str() {
            "add_part_rect" => {
                let name = op
                    .args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("pouch_body");
                let w = num(op.args.get("w_mm").ok_or_else(|| {
                    WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing w_mm")
                })?)?;
                let h = num(op.args.get("h_mm").ok_or_else(|| {
                    WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing h_mm")
                })?)?;
                let part_id = format!("part:{tpl_id}:{name}");
                let mut feats = vec![];
                let dia = 1.0;
                let x0 = seam_allowance_mm;
                let y0 = seam_allowance_mm;
                let x1 = w - seam_allowance_mm;
                let y1 = h - seam_allowance_mm;

                if x1 <= x0 || y1 <= y0 {
                    return Err(WizardReason::new(
                        WizardReasonCode::WizardInputInvalid,
                        "seam allowance too large for pouch size",
                    ));
                }
                if hole_pitch_mm <= 0.0 {
                    return Err(WizardReason::new(
                        WizardReasonCode::WizardInputInvalid,
                        "hole_pitch_mm must be > 0",
                    ));
                }

                let mut x = x0;
                while x <= x1 {
                    feats.push(Feature2D::StitchHole {
                        x_mm: x,
                        y_mm: y0,
                        diameter_mm: dia,
                    });
                    x += hole_pitch_mm;
                    if feats.len() > 10_000 {
                        return Err(WizardReason::new(
                            WizardReasonCode::WizardInputInvalid,
                            "too many stitch holes",
                        ));
                    }
                }
                let mut y = y0 + hole_pitch_mm;
                while y <= y1 {
                    feats.push(Feature2D::StitchHole {
                        x_mm: x1,
                        y_mm: y,
                        diameter_mm: dia,
                    });
                    y += hole_pitch_mm;
                    if feats.len() > 10_000 {
                        return Err(WizardReason::new(
                            WizardReasonCode::WizardInputInvalid,
                            "too many stitch holes",
                        ));
                    }
                }
                let mut x = x1 - hole_pitch_mm;
                while x >= x0 {
                    feats.push(Feature2D::StitchHole {
                        x_mm: x,
                        y_mm: y1,
                        diameter_mm: dia,
                    });
                    if x < x0 + hole_pitch_mm {
                        break;
                    }
                    x -= hole_pitch_mm;
                    if feats.len() > 10_000 {
                        return Err(WizardReason::new(
                            WizardReasonCode::WizardInputInvalid,
                            "too many stitch holes",
                        ));
                    }
                }
                let mut y = y1 - hole_pitch_mm;
                while y >= y0 + hole_pitch_mm {
                    feats.push(Feature2D::StitchHole {
                        x_mm: x0,
                        y_mm: y,
                        diameter_mm: dia,
                    });
                    if y < y0 + 2.0 * hole_pitch_mm {
                        break;
                    }
                    y -= hole_pitch_mm;
                    if feats.len() > 10_000 {
                        return Err(WizardReason::new(
                            WizardReasonCode::WizardInputInvalid,
                            "too many stitch holes",
                        ));
                    }
                }

                parts.push(PartDraft {
                    part_id: part_id.clone(),
                    name: name.to_string(),
                    outline: Outline2D::Rect { w_mm: w, h_mm: h },
                    features: feats,
                    qty: 1,
                    tags: vec!["wizard".into(), "leather_pouch".into()],
                });

                annotations.push(AnnotationDraft::StitchPitch {
                    part_id: part_id.clone(),
                    pitch_mm: hole_pitch_mm,
                });
                annotations.push(AnnotationDraft::SeamAllowance {
                    part_id,
                    allowance_mm: seam_allowance_mm,
                });
            }
            "recommend_nest" => {
                nest = Some(NestJobDraft {
                    material_preset_id: material_id.to_string(),
                    process_preset_id: process_id.to_string(),
                    kerf_mm,
                    margin_mm,
                    seed,
                    allow_rotate: true,
                    grain: grain.to_string(),
                });
            }
            _ => {}
        }
    }

    let mut d = PartsDraft {
        schema_version: 1,
        parts,
        annotations,
        recommended_nest_job: nest,
    };
    d = normalize(d);
    validate_parts(&d)?;
    Ok(d)
}
