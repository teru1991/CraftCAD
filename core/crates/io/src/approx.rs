use crate::model::*;
use crate::options::ImportOptions;
use crate::reasons::{AppError, ReasonCode};
use crate::report::IoReport;

fn dist(a: Point2D, b: Point2D) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn lerp(a: Point2D, b: Point2D, t: f64) -> Point2D {
    Point2D {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

fn cubic_point(a: Point2D, c1: Point2D, c2: Point2D, b: Point2D, t: f64) -> Point2D {
    let p0 = lerp(a, c1, t);
    let p1 = lerp(c1, c2, t);
    let p2 = lerp(c2, b, t);
    let q0 = lerp(p0, p1, t);
    let q1 = lerp(p1, p2, t);
    lerp(q0, q1, t)
}

fn clamp_usize(v: usize, lo: usize, hi: usize) -> usize {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

fn flatten_arc(
    center: Point2D,
    radius: f64,
    start: f64,
    end: f64,
    ccw: bool,
    opts: &ImportOptions,
) -> (Vec<Point2D>, usize, bool) {
    let a0 = start;
    let mut a1 = end;
    if ccw {
        while a1 < a0 {
            a1 += std::f64::consts::TAU;
        }
    } else {
        while a1 > a0 {
            a1 -= std::f64::consts::TAU;
        }
    }
    let delta = (a1 - a0).abs();

    let eps = opts.determinism.approx_eps.max(1e-15);
    let arc_len = radius.abs() * delta;
    let mut seg = ((arc_len / eps).ceil() as usize).max(1);
    seg = clamp_usize(
        seg,
        opts.determinism.approx_min_segments,
        opts.determinism.approx_max_segments,
    );

    let mut pts = Vec::with_capacity(seg + 1);
    for i in 0..=seg {
        let t = (i as f64) / (seg as f64);
        let ang = a0 + (a1 - a0) * t;
        pts.push(Point2D {
            x: center.x + radius * ang.cos(),
            y: center.y + radius * ang.sin(),
        });
    }

    let clamped =
        seg == opts.determinism.approx_min_segments || seg == opts.determinism.approx_max_segments;
    (pts, seg, clamped)
}

fn flatten_cubic(
    a: Point2D,
    c1: Point2D,
    c2: Point2D,
    b: Point2D,
    opts: &ImportOptions,
) -> (Vec<Point2D>, usize, bool) {
    let eps = opts.determinism.approx_eps.max(1e-15);
    let approx_len = dist(a, c1) + dist(c1, c2) + dist(c2, b);
    let mut seg = ((approx_len / eps).ceil() as usize).max(1);
    seg = clamp_usize(
        seg,
        opts.determinism.approx_min_segments,
        opts.determinism.approx_max_segments,
    );

    let mut pts = Vec::with_capacity(seg + 1);
    for i in 0..=seg {
        let t = (i as f64) / (seg as f64);
        pts.push(cubic_point(a, c1, c2, b, t));
    }
    let clamped =
        seg == opts.determinism.approx_min_segments || seg == opts.determinism.approx_max_segments;
    (pts, seg, clamped)
}

fn polyline_from_points(points: &[Point2D]) -> Vec<Segment2D> {
    let mut segs = Vec::new();
    for w in points.windows(2) {
        segs.push(Segment2D::Line { a: w[0], b: w[1] });
    }
    segs
}

pub fn apply_approx(
    model: &mut InternalModel,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
    report: &mut IoReport,
) {
    if !opts.enable_approx {
        return;
    }

    let mut new_entities = Vec::with_capacity(model.entities.len());

    for e in model.entities.drain(..) {
        match e {
            Entity::Path(mut p) => {
                let mut out_segments: Vec<Segment2D> = Vec::new();

                for s in p.segments.drain(..) {
                    match s {
                        Segment2D::CubicBezier { a, c1, c2, b } => {
                            let (pts, seg, clamped) = flatten_cubic(a, c1, c2, b, opts);
                            out_segments.extend(polyline_from_points(&pts));
                            report.approx_applied_count += 1;
                            warnings.push(
                                AppError::new(
                                    ReasonCode::IO_CURVE_APPROX_APPLIED,
                                    "cubic bezier approximated to polyline",
                                )
                                .with_context("id", p.id.clone())
                                .with_context("method", "spline_flatten")
                                .with_context("eps_used", opts.determinism.approx_eps.to_string())
                                .with_context("segments", seg.to_string())
                                .with_context("clamped", clamped.to_string()),
                            );
                        }
                        Segment2D::Arc {
                            center,
                            radius,
                            start_rad,
                            end_rad,
                            ccw,
                        } => {
                            let (pts, seg, clamped) =
                                flatten_arc(center, radius, start_rad, end_rad, ccw, opts);
                            out_segments.extend(polyline_from_points(&pts));
                            report.approx_applied_count += 1;
                            warnings.push(
                                AppError::new(
                                    ReasonCode::IO_CURVE_APPROX_APPLIED,
                                    "arc approximated to polyline",
                                )
                                .with_context("id", p.id.clone())
                                .with_context("method", "arc_flatten")
                                .with_context("eps_used", opts.determinism.approx_eps.to_string())
                                .with_context("segments", seg.to_string())
                                .with_context("clamped", clamped.to_string()),
                            );
                        }
                        Segment2D::Circle { center, radius } => {
                            let (pts, seg, clamped) =
                                flatten_arc(center, radius, 0.0, std::f64::consts::TAU, true, opts);
                            out_segments.extend(polyline_from_points(&pts));
                            report.approx_applied_count += 1;
                            warnings.push(
                                AppError::new(
                                    ReasonCode::IO_CURVE_APPROX_APPLIED,
                                    "circle approximated to polyline",
                                )
                                .with_context("id", p.id.clone())
                                .with_context("method", "arc_flatten")
                                .with_context("eps_used", opts.determinism.approx_eps.to_string())
                                .with_context("segments", seg.to_string())
                                .with_context("clamped", clamped.to_string()),
                            );
                        }
                        other => {
                            out_segments.push(other);
                        }
                    }
                }

                p.segments = out_segments;
                new_entities.push(Entity::Path(p));
            }
            other => new_entities.push(other),
        }
    }

    model.entities = new_entities;
}
