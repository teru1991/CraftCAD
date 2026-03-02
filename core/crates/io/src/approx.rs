use crate::model::*;
use crate::reasons::{AppError, ReasonCode};

fn lerp(a: Point2D, b: Point2D, t: f64) -> Point2D {
    Point2D {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

fn cubic(a: Point2D, c1: Point2D, c2: Point2D, b: Point2D, t: f64) -> Point2D {
    let p0 = lerp(a, c1, t);
    let p1 = lerp(c1, c2, t);
    let p2 = lerp(c2, b, t);
    let q0 = lerp(p0, p1, t);
    let q1 = lerp(p1, p2, t);
    lerp(q0, q1, t)
}

pub fn cubic_to_polyline(
    a: Point2D,
    c1: Point2D,
    c2: Point2D,
    b: Point2D,
    seg: usize,
) -> Vec<Point2D> {
    let seg = seg.max(2);
    let mut pts = Vec::with_capacity(seg + 1);
    for i in 0..=seg {
        let t = i as f64 / seg as f64;
        pts.push(cubic(a, c1, c2, b, t));
    }
    pts
}

pub fn apply_approx(model: &mut InternalModel, seg: usize, warnings: &mut Vec<AppError>) {
    for ent in &mut model.entities {
        if let Entity::Path(p) = ent {
            let mut new_segments: Vec<Segment2D> = Vec::new();
            for s in &p.segments {
                match s {
                    Segment2D::CubicBezier { a, c1, c2, b } => {
                        let pts = cubic_to_polyline(*a, *c1, *c2, *b, seg);
                        for w in pts.windows(2) {
                            new_segments.push(Segment2D::Line { a: w[0], b: w[1] });
                        }
                        warnings.push(
                            AppError::new(
                                ReasonCode::IO_CURVE_APPROX_APPLIED,
                                "cubic bezier approximated to polyline",
                            )
                            .with_context("path_id", p.id.clone())
                            .with_context("segments", seg.to_string()),
                        );
                    }
                    _ => new_segments.push(s.clone()),
                }
            }
            p.segments = new_segments;
        }
    }
}
