use crate::model::*;
use crate::reasons::{AppError, ReasonCode};

fn round_step(x: f64, step: f64) -> f64 {
    if !x.is_finite() || step <= 0.0 {
        return x;
    }
    (x / step).round() * step
}

fn round_point(p: Point2D, step: f64) -> Point2D {
    Point2D {
        x: round_step(p.x, step),
        y: round_step(p.y, step),
    }
}

fn sanitize_point(p: Point2D) -> Option<Point2D> {
    if p.is_finite() {
        Some(p)
    } else {
        None
    }
}

pub fn normalize_model(model: &mut InternalModel, step: f64, warnings: &mut Vec<AppError>) {
    for ent in &mut model.entities {
        if let Entity::Path(p) = ent {
            let mut changed = false;
            p.segments.retain_mut(|seg| match seg {
                Segment2D::Line { a, b } => {
                    let Some(aa) = sanitize_point(*a) else {
                        changed = true;
                        return false;
                    };
                    let Some(bb) = sanitize_point(*b) else {
                        changed = true;
                        return false;
                    };
                    *a = round_point(aa, step);
                    *b = round_point(bb, step);
                    true
                }
                Segment2D::CubicBezier { a, c1, c2, b } => {
                    let Some(aa) = sanitize_point(*a) else {
                        changed = true;
                        return false;
                    };
                    let Some(cc1) = sanitize_point(*c1) else {
                        changed = true;
                        return false;
                    };
                    let Some(cc2) = sanitize_point(*c2) else {
                        changed = true;
                        return false;
                    };
                    let Some(bb) = sanitize_point(*b) else {
                        changed = true;
                        return false;
                    };
                    *a = round_point(aa, step);
                    *c1 = round_point(cc1, step);
                    *c2 = round_point(cc2, step);
                    *b = round_point(bb, step);
                    true
                }
                Segment2D::Arc {
                    center,
                    radius,
                    start_rad,
                    end_rad,
                    ..
                } => {
                    let Some(c) = sanitize_point(*center) else {
                        changed = true;
                        return false;
                    };
                    if !radius.is_finite() || !start_rad.is_finite() || !end_rad.is_finite() {
                        changed = true;
                        return false;
                    }
                    *center = round_point(c, step);
                    *radius = round_step(*radius, step);
                    *start_rad = round_step(*start_rad, step);
                    *end_rad = round_step(*end_rad, step);
                    true
                }
                Segment2D::Circle { center, radius } => {
                    let Some(c) = sanitize_point(*center) else {
                        changed = true;
                        return false;
                    };
                    if !radius.is_finite() {
                        changed = true;
                        return false;
                    }
                    *center = round_point(c, step);
                    *radius = round_step(*radius, step);
                    true
                }
            });

            if changed {
                warnings.push(
                    AppError::new(
                        ReasonCode::IO_SANITIZE_NONFINITE,
                        "non-finite coordinates were dropped during normalization",
                    )
                    .with_context("path_id", p.id.clone()),
                );
            }
        } else if let Entity::Text(t) = ent {
            if t.pos.is_finite() {
                t.pos = round_point(t.pos, step);
            } else {
                warnings.push(
                    AppError::new(
                        ReasonCode::IO_SANITIZE_NONFINITE,
                        "text position had non-finite coordinate; set to (0,0)",
                    )
                    .with_context("text_id", t.id.clone()),
                );
                t.pos = Point2D { x: 0.0, y: 0.0 };
            }
        }
    }
}
