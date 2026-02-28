use crate::{util::dist2, EpsilonPolicy, Geom2D, IntersectionSet, Vec2};
use craftcad_serialize::{Reason, ReasonCode, Result};

trait ReasonExt {
    fn with_debug(self, k: &str, v: impl Into<serde_json::Value>) -> Reason;
}
impl ReasonExt for Reason {
    fn with_debug(mut self, k: &str, v: impl Into<serde_json::Value>) -> Reason {
        self.debug.insert(k.to_string(), v.into());
        self
    }
}

fn line_line(
    a0: Vec2,
    a1: Vec2,
    b0: Vec2,
    b1: Vec2,
    eps: &EpsilonPolicy,
) -> Result<(Vec2, f64, f64)> {
    let r = Vec2 {
        x: a1.x - a0.x,
        y: a1.y - a0.y,
    };
    let s = Vec2 {
        x: b1.x - b0.x,
        y: b1.y - b0.y,
    };
    let denom = r.x * s.y - r.y * s.x;
    let qp = Vec2 {
        x: b0.x - a0.x,
        y: b0.y - a0.y,
    };
    let qpxr = qp.x * r.y - qp.y * r.x;

    if denom.abs() <= eps.intersect_tol {
        if qpxr.abs() <= eps.intersect_tol {
            return Err(Reason::from_code(ReasonCode::GeomIntersectionAmbiguous)
                .with_debug("case", "colinear_overlap"));
        }
        return Err(Reason::from_code(ReasonCode::GeomNoIntersection)
            .with_debug("denom", denom)
            .with_debug("qpxr", qpxr));
    }

    let t = (qp.x * s.y - qp.y * s.x) / denom;
    let u = (qp.x * r.y - qp.y * r.x) / denom;
    if t < -eps.intersect_tol
        || t > 1.0 + eps.intersect_tol
        || u < -eps.intersect_tol
        || u > 1.0 + eps.intersect_tol
    {
        return Err(Reason::from_code(ReasonCode::GeomNoIntersection)
            .with_debug("t", t)
            .with_debug("u", u));
    }
    let p = Vec2 {
        x: a0.x + t * r.x,
        y: a0.y + t * r.y,
    };
    Ok((p, t, u))
}

pub fn intersect(a: &Geom2D, b: &Geom2D, eps: &EpsilonPolicy) -> Result<IntersectionSet> {
    match (a, b) {
        (Geom2D::Line { a: a0, b: a1 }, Geom2D::Line { a: b0, b: b1 }) => {
            let (p, t, u) = line_line(*a0, *a1, *b0, *b1, eps)?;
            Ok(IntersectionSet {
                points: vec![p],
                ambiguous: false,
                debug: serde_json::json!({"t":t,"u":u}),
            })
        }
        (Geom2D::Polyline { pts, closed }, Geom2D::Line { a: b0, b: b1 })
        | (Geom2D::Line { a: b0, b: b1 }, Geom2D::Polyline { pts, closed }) => {
            let seg_count = if *closed {
                pts.len()
            } else {
                pts.len().saturating_sub(1)
            };
            let mut out: Vec<Vec2> = Vec::new();
            for i in 0..seg_count {
                let p0 = pts[i];
                let p1 = if i + 1 < pts.len() {
                    pts[i + 1]
                } else {
                    pts[0]
                };
                if let Ok((p, _, _)) = line_line(p0, p1, *b0, *b1, eps) {
                    if !out
                        .iter()
                        .any(|q| dist2(*q, p) <= eps.eq_dist * eps.eq_dist)
                    {
                        out.push(p);
                    }
                }
            }
            if out.is_empty() {
                return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
            }
            let ambiguous = out.len() >= 2;
            Ok(IntersectionSet {
                points: out.clone(),
                ambiguous,
                debug: serde_json::json!({"candidate_count": out.len(), "points_preview": out.iter().take(5).collect::<Vec<_>>() }),
            })
        }
        _ => {
            Err(Reason::from_code(ReasonCode::GeomNoIntersection)
                .with_debug("unsupported_pair", true))
        }
    }
}
