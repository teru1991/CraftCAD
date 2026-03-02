use crate::parts::model::*;
use crate::reasons::{WizardReason, WizardReasonCode};

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

pub fn validate_parts(parts: &PartsDraft) -> Result<(), WizardReason> {
    if parts.schema_version < 1 {
        return Err(WizardReason::new(
            WizardReasonCode::WizardTemplateInvalid,
            "PartsDraft.schema_version must be >=1",
        ));
    }
    if parts.parts.is_empty() {
        return Err(WizardReason::new(
            WizardReasonCode::WizardDslInvalid,
            "no parts generated",
        ));
    }

    for p in &parts.parts {
        if p.qty <= 0 {
            return Err(WizardReason::new(
                WizardReasonCode::WizardInputInvalid,
                format!("part qty must be >0: {}", p.part_id),
            ));
        }
        if p.part_id.is_empty() {
            return Err(WizardReason::new(
                WizardReasonCode::WizardDslInvalid,
                "part_id empty",
            ));
        }

        let (w, h) = match p.outline {
            Outline2D::Rect { w_mm, h_mm } => {
                if !(w_mm > 0.0 && h_mm > 0.0) {
                    return Err(WizardReason::new(
                        WizardReasonCode::WizardInputInvalid,
                        format!("outline must be >0: {}", p.part_id),
                    ));
                }
                (w_mm, h_mm)
            }
        };

        for f in &p.features {
            match *f {
                Feature2D::HoleCircle {
                    cx_mm,
                    cy_mm,
                    diameter_mm,
                } => {
                    if diameter_mm <= 0.0 {
                        return Err(WizardReason::new(
                            WizardReasonCode::WizardInputInvalid,
                            "hole diameter must be >0",
                        ));
                    }
                    let r = diameter_mm / 2.0;
                    if cx_mm - r < 0.0 || cy_mm - r < 0.0 || cx_mm + r > w || cy_mm + r > h {
                        return Err(WizardReason::new(
                            WizardReasonCode::WizardInputInvalid,
                            format!(
                                "hole out of bounds: part={} cx={} cy={} d={}",
                                p.part_id,
                                round3(cx_mm),
                                round3(cy_mm),
                                round3(diameter_mm)
                            ),
                        ));
                    }
                }
                Feature2D::StitchHole {
                    x_mm,
                    y_mm,
                    diameter_mm,
                } => {
                    if diameter_mm <= 0.0 {
                        return Err(WizardReason::new(
                            WizardReasonCode::WizardInputInvalid,
                            "stitch hole diameter must be >0",
                        ));
                    }
                    if x_mm < 0.0 || y_mm < 0.0 || x_mm > w || y_mm > h {
                        return Err(WizardReason::new(
                            WizardReasonCode::WizardInputInvalid,
                            "stitch hole out of bounds",
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}
