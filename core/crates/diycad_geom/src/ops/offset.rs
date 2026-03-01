use crate::{
    util::{add, dist2, mul, sub},
    EpsilonPolicy, Geom2D, Vec2,
};
use craftcad_serialize::{Reason, ReasonCode, Result};

fn line_normal(a: Vec2, b: Vec2, eps: &EpsilonPolicy) -> Result<Vec2> {
    let d = sub(b, a);
    let len2 = d.x * d.x + d.y * d.y;
    if len2 <= eps.eq_dist * eps.eq_dist {
        return Err(Reason::from_code(ReasonCode::GeomDegenerate));
    }
    let inv = 1.0 / len2.sqrt();
    Ok(Vec2 {
        x: -d.y * inv,
        y: d.x * inv,
    })
}

fn line_intersection_inf(
    a0: Vec2,
    a1: Vec2,
    b0: Vec2,
    b1: Vec2,
    eps: &EpsilonPolicy,
) -> Result<Vec2> {
    let r = sub(a1, a0);
    let s = sub(b1, b0);
    let denom = r.x * s.y - r.y * s.x;
    if denom.abs() <= eps.intersect_tol {
        return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported));
    }
    let qp = sub(b0, a0);
    let t = (qp.x * s.y - qp.y * s.x) / denom;
    Ok(add(a0, mul(r, t)))
}

pub fn offset(g: &Geom2D, dist: f64, eps: &EpsilonPolicy) -> Result<Geom2D> {
    if !dist.is_finite() {
        return Err(Reason::from_code(ReasonCode::GeomInvalidNumeric));
    }
    match g {
        Geom2D::Line { a, b } => {
            let n = line_normal(*a, *b, eps)?;
            let d = mul(n, dist);
            Ok(Geom2D::Line {
                a: add(*a, d),
                b: add(*b, d),
            })
        }
        Geom2D::Polyline { pts, closed } => {
            if *closed {
                return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported));
            }
            if pts.len() < 2 {
                return Err(Reason::from_code(ReasonCode::GeomDegenerate));
            }
            let seg_count = pts.len() - 1;
            let mut segs = Vec::with_capacity(seg_count);
            for i in 0..seg_count {
                let a = pts[i];
                let b = pts[i + 1];
                let n = line_normal(a, b, eps)?;
                let d = mul(n, dist);
                segs.push((add(a, d), add(b, d)));
            }

            let mut out = Vec::with_capacity(pts.len());
            out.push(segs[0].0);
            for i in 1..seg_count {
                let (a0, a1) = segs[i - 1];
                let (b0, b1) = segs[i];
                let j = line_intersection_inf(a0, a1, b0, b1, eps)?;
                out.push(j);
            }
            out.push(segs[seg_count - 1].1);

            for i in 0..out.len().saturating_sub(1) {
                if dist2(out[i], out[i + 1]) <= eps.eq_dist * eps.eq_dist {
                    return Err(Reason::from_code(ReasonCode::GeomOffsetSelfIntersection));
                }
            }

            Ok(Geom2D::Polyline {
                pts: out,
                closed: false,
            })
        }
        _ => Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
    }
}
