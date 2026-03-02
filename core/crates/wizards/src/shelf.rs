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
    qty: i32,
) -> Result<PartsDraft, WizardReason> {
    let mut parts: Vec<PartDraft> = vec![];
    let mut annotations: Vec<AnnotationDraft> = vec![];
    let mut nest: Option<NestJobDraft> = None;

    for op in ops {
        if op.op != "add_part_rect" {
            continue;
        }
        let name = op
            .args
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("shelf_board");
        let w = num(op.args.get("w_mm").ok_or_else(|| {
            WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing w_mm")
        })?)?;
        let h = num(op.args.get("h_mm").ok_or_else(|| {
            WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing h_mm")
        })?)?;
        let part_id = format!("part:{tpl_id}:{name}");
        parts.push(PartDraft {
            part_id,
            name: name.to_string(),
            outline: Outline2D::Rect { w_mm: w, h_mm: h },
            features: vec![],
            qty,
            tags: vec!["wizard".into(), "shelf".into()],
        });
    }

    for op in ops {
        match op.op.as_str() {
            "add_hole_circle_array" => {
                let part_name = op
                    .args
                    .get("part_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing part_name")
                    })?;
                let part_id = format!("part:{tpl_id}:{part_name}");
                let diam = num(op.args.get("diameter_mm").ok_or_else(|| {
                    WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing diameter_mm")
                })?)?;
                let off = num(op.args.get("offset_mm").ok_or_else(|| {
                    WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing offset_mm")
                })?)?;
                let pattern = op
                    .args
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .unwrap_or("4_corners");

                let p = parts
                    .iter_mut()
                    .find(|p| p.part_id == part_id)
                    .ok_or_else(|| {
                        WizardReason::new(
                            WizardReasonCode::WizardDslInvalid,
                            "target part not found",
                        )
                    })?;
                let (w, h) = match p.outline {
                    Outline2D::Rect { w_mm, h_mm } => (w_mm, h_mm),
                };

                let coords = match pattern {
                    "4_corners" => vec![
                        (off, off),
                        (w - off, off),
                        (off, h - off),
                        (w - off, h - off),
                    ],
                    _ => {
                        return Err(WizardReason::new(
                            WizardReasonCode::WizardDslInvalid,
                            format!("unsupported hole pattern: {pattern}"),
                        ));
                    }
                };

                for (cx, cy) in &coords {
                    p.features.push(Feature2D::HoleCircle {
                        cx_mm: *cx,
                        cy_mm: *cy,
                        diameter_mm: diam,
                    });
                }

                annotations.push(AnnotationDraft::HoleCallout {
                    part_id: p.part_id.clone(),
                    diameter_mm: diam,
                    count: coords.len() as i32,
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
