use crate::model::*;
use crate::options::ImportOptions;
use crate::reasons::{AppError, ReasonCode};
use crate::report::IoReport;

use sha2::{Digest, Sha256};

fn round_to_step(v: f64, step: f64) -> f64 {
    if !v.is_finite() {
        return v;
    }
    if step <= 0.0 {
        return v;
    }
    (v / step).round() * step
}

fn stable_hash_entity(e: &Entity, round_step: f64) -> String {
    let mut hasher = Sha256::new();

    match e {
        Entity::Path(p) => {
            hasher.update(b"path");
            hasher.update(p.stroke.layer.as_bytes());
            hasher.update(p.stroke.linetype.as_bytes());
            hasher.update(if p.closed { b"1" } else { b"0" });

            for s in &p.segments {
                match s {
                    Segment2D::Line { a, b } => {
                        hasher.update(b"l");
                        for v in [a.x, a.y, b.x, b.y] {
                            let r = round_to_step(v, round_step);
                            hasher.update(r.to_le_bytes());
                        }
                    }
                    Segment2D::Arc {
                        center,
                        radius,
                        start_rad,
                        end_rad,
                        ccw,
                    } => {
                        hasher.update(b"a");
                        for v in [center.x, center.y, *radius, *start_rad, *end_rad] {
                            let r = round_to_step(v, round_step);
                            hasher.update(r.to_le_bytes());
                        }
                        hasher.update(if *ccw { b"1" } else { b"0" });
                    }
                    Segment2D::Circle { center, radius } => {
                        hasher.update(b"c");
                        for v in [center.x, center.y, *radius] {
                            let r = round_to_step(v, round_step);
                            hasher.update(r.to_le_bytes());
                        }
                    }
                    Segment2D::CubicBezier { a, c1, c2, b } => {
                        hasher.update(b"b");
                        for v in [a.x, a.y, c1.x, c1.y, c2.x, c2.y, b.x, b.y] {
                            let r = round_to_step(v, round_step);
                            hasher.update(r.to_le_bytes());
                        }
                    }
                }
            }
        }
        Entity::Text(t) => {
            hasher.update(b"text");
            hasher.update(t.layer.as_bytes());
            hasher.update(t.text.as_bytes());
            for v in [t.pos.x, t.pos.y, t.rotation_rad, t.size as f64] {
                let r = round_to_step(v, round_step);
                hasher.update(r.to_le_bytes());
            }
        }
    }

    hex::encode(hasher.finalize())
}

fn endpoint_distance(p: &PathEntity) -> Option<f64> {
    let first = p.segments.first()?;
    let last = p.segments.last()?;

    let a = match first {
        Segment2D::Line { a, .. } => *a,
        Segment2D::Arc { .. } => return None,
        Segment2D::Circle { .. } => return None,
        Segment2D::CubicBezier { a, .. } => *a,
    };
    let b = match last {
        Segment2D::Line { b, .. } => *b,
        Segment2D::Arc { .. } => return None,
        Segment2D::Circle { .. } => return None,
        Segment2D::CubicBezier { b, .. } => *b,
    };

    if !a.is_finite() || !b.is_finite() {
        return None;
    }
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    Some((dx * dx + dy * dy).sqrt())
}

fn round_point(p: &mut Point2D, step: f64) -> bool {
    let before = *p;
    p.x = round_to_step(p.x, step);
    p.y = round_to_step(p.y, step);
    (before.x != p.x) || (before.y != p.y)
}

fn sanitize_and_round_entity(
    e: &mut Entity,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
) -> bool {
    let mut dropped = false;
    let mut rounded_any = false;

    match e {
        Entity::Path(p) => {
            for s in &p.segments {
                if !s.is_finite() {
                    warnings.push(
                        AppError::new(
                            ReasonCode::IoSanitizeNonfinite,
                            "non-finite values detected; entity dropped",
                        )
                        .with_context("entity_kind", "path")
                        .with_context("id", p.id.clone()),
                    );
                    dropped = true;
                    break;
                }
            }
            if dropped {
                return true;
            }

            for s in &mut p.segments {
                match s {
                    Segment2D::Line { a, b } => {
                        rounded_any |= round_point(a, opts.determinism.round_step);
                        rounded_any |= round_point(b, opts.determinism.round_step);
                    }
                    Segment2D::Arc {
                        center,
                        radius,
                        start_rad,
                        end_rad,
                        ..
                    } => {
                        rounded_any |= round_point(center, opts.determinism.round_step);
                        let r0 = *radius;
                        *radius = round_to_step(*radius, opts.determinism.round_step);
                        rounded_any |= r0 != *radius;

                        let s0 = *start_rad;
                        *start_rad = round_to_step(*start_rad, opts.determinism.round_step);
                        rounded_any |= s0 != *start_rad;

                        let e0 = *end_rad;
                        *end_rad = round_to_step(*end_rad, opts.determinism.round_step);
                        rounded_any |= e0 != *end_rad;
                    }
                    Segment2D::Circle { center, radius } => {
                        rounded_any |= round_point(center, opts.determinism.round_step);
                        let r0 = *radius;
                        *radius = round_to_step(*radius, opts.determinism.round_step);
                        rounded_any |= r0 != *radius;
                    }
                    Segment2D::CubicBezier { a, c1, c2, b } => {
                        rounded_any |= round_point(a, opts.determinism.round_step);
                        rounded_any |= round_point(c1, opts.determinism.round_step);
                        rounded_any |= round_point(c2, opts.determinism.round_step);
                        rounded_any |= round_point(b, opts.determinism.round_step);
                    }
                }
            }

            if let Some(d) = endpoint_distance(p) {
                if d <= opts.determinism.close_eps && !p.closed {
                    p.closed = true;
                    warnings.push(
                        AppError::new(ReasonCode::IoPathClosedByEps, "path closed by eps")
                            .with_context("id", p.id.clone())
                            .with_context("close_eps", opts.determinism.close_eps.to_string())
                            .with_context("distance", d.to_string()),
                    );
                }
            }
        }
        Entity::Text(t) => {
            if !t.pos.is_finite() || !t.rotation_rad.is_finite() || !(t.size as f64).is_finite() {
                warnings.push(
                    AppError::new(
                        ReasonCode::IoSanitizeNonfinite,
                        "non-finite values detected; text dropped",
                    )
                    .with_context("entity_kind", "text")
                    .with_context("id", t.id.clone()),
                );
                dropped = true;
            } else {
                rounded_any |= round_point(&mut t.pos, opts.determinism.round_step);
                let r0 = t.rotation_rad;
                t.rotation_rad = round_to_step(t.rotation_rad, opts.determinism.round_step);
                rounded_any |= r0 != t.rotation_rad;
            }
        }
    }

    if rounded_any {
        warnings.push(
            AppError::new(
                ReasonCode::IoNormalizeRounded,
                "coordinates rounded by round_step",
            )
            .with_context("round_step", opts.determinism.round_step.to_string()),
        );
    }

    dropped
}

pub fn normalize_model(
    model: &mut InternalModel,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
    report: &mut IoReport,
) {
    report.entities_in = model.entities.len();
    report.texts_in = model.texts.len();

    let mut kept: Vec<Entity> = Vec::with_capacity(model.entities.len());
    for mut e in model.entities.drain(..) {
        let drop = sanitize_and_round_entity(&mut e, opts, warnings);
        if !drop {
            kept.push(e);
        }
    }
    model.entities = kept;

    let step = opts.determinism.round_step;
    model.entities.sort_by(|a, b| {
        let la = a.layer_key();
        let lb = b.layer_key();
        let ka = a.kind_key();
        let kb = b.kind_key();

        let bba = a.bbox();
        let bbb = b.bbox();

        let ax = if bba.is_valid() { bba.min.x } else { 0.0 };
        let ay = if bba.is_valid() { bba.min.y } else { 0.0 };
        let bx = if bbb.is_valid() { bbb.min.x } else { 0.0 };
        let by = if bbb.is_valid() { bbb.min.y } else { 0.0 };

        let ha = stable_hash_entity(a, step);
        let hb = stable_hash_entity(b, step);

        (la, ka, ax.to_bits(), ay.to_bits(), ha).cmp(&(lb, kb, bx.to_bits(), by.to_bits(), hb))
    });

    model.texts.sort_by(|a, b| {
        let ba = a.bbox();
        let bb = b.bbox();
        let ax = if ba.is_valid() { ba.min.x } else { 0.0 };
        let ay = if ba.is_valid() { ba.min.y } else { 0.0 };
        let bx = if bb.is_valid() { bb.min.x } else { 0.0 };
        let by = if bb.is_valid() { bb.min.y } else { 0.0 };

        (a.layer.as_str(), ax.to_bits(), ay.to_bits(), a.id.as_str()).cmp(&(
            b.layer.as_str(),
            bx.to_bits(),
            by.to_bits(),
            b.id.as_str(),
        ))
    });

    report.entities_out = model.entities.len();
    report.texts_out = model.texts.len();
}
