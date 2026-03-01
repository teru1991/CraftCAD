use crate::{Geom2D, Vec2};
use craftcad_serialize::{Reason, ReasonCode, Result};

fn sub(a: Vec2, b: Vec2) -> Vec2 {
    Vec2 {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}
fn add(a: Vec2, b: Vec2) -> Vec2 {
    Vec2 {
        x: a.x + b.x,
        y: a.y + b.y,
    }
}
fn mul(a: Vec2, s: f64) -> Vec2 {
    Vec2 {
        x: a.x * s,
        y: a.y * s,
    }
}
fn dot(a: Vec2, b: Vec2) -> f64 {
    a.x * b.x + a.y * b.y
}
fn len(a: Vec2) -> f64 {
    (a.x * a.x + a.y * a.y).sqrt()
}
fn norm(a: Vec2) -> Option<Vec2> {
    let l = len(a);
    if l <= 1e-12 {
        None
    } else {
        Some(Vec2 {
            x: a.x / l,
            y: a.y / l,
        })
    }
}

pub fn mirror_geom(geom: &Geom2D, axis_a: Vec2, axis_b: Vec2) -> Result<Geom2D> {
    let dir = sub(axis_b, axis_a);
    let u = norm(dir).ok_or_else(|| Reason::from_code(ReasonCode::EditMirrorAxisInvalid))?;
    let mirror_pt = |p: Vec2| {
        let ap = sub(p, axis_a);
        let proj = mul(u, dot(ap, u));
        let perp = sub(ap, proj);
        add(axis_a, sub(proj, perp))
    };
    Ok(match geom {
        Geom2D::Line { a, b } => Geom2D::Line {
            a: mirror_pt(*a),
            b: mirror_pt(*b),
        },
        Geom2D::Circle { c, r } => Geom2D::Circle {
            c: mirror_pt(*c),
            r: *r,
        },
        Geom2D::Arc {
            c,
            r,
            start_angle,
            end_angle,
            ccw,
        } => Geom2D::Arc {
            c: mirror_pt(*c),
            r: *r,
            start_angle: -*start_angle,
            end_angle: -*end_angle,
            ccw: !*ccw,
        },
        Geom2D::Polyline { pts, closed } => Geom2D::Polyline {
            pts: pts.iter().copied().map(mirror_pt).collect(),
            closed: *closed,
        },
    })
}

pub fn chamfer_lines(
    a: Vec2,
    b: Vec2,
    c: Vec2,
    d: Vec2,
    dist: f64,
) -> Result<(Geom2D, Geom2D, Geom2D)> {
    if !dist.is_finite() || dist <= 0.0 {
        return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
    }
    let p = line_intersection(a, b, c, d)
        .ok_or_else(|| Reason::from_code(ReasonCode::GeomNoIntersection))?;
    let ua = norm(sub(a, p)).ok_or_else(|| Reason::from_code(ReasonCode::GeomDegenerate))?;
    let ub = norm(sub(c, p)).ok_or_else(|| Reason::from_code(ReasonCode::GeomDegenerate))?;
    let pa = add(p, mul(ua, dist));
    let pb = add(p, mul(ub, dist));
    if len(sub(pa, p)) < 1e-12 || len(sub(pb, p)) < 1e-12 {
        return Err(Reason::from_code(ReasonCode::EditChamferDistanceTooLarge));
    }
    Ok((
        Geom2D::Line { a, b: pa },
        Geom2D::Line { a: c, b: pb },
        Geom2D::Line { a: pa, b: pb },
    ))
}

pub fn fillet_lines(
    a: Vec2,
    b: Vec2,
    c: Vec2,
    d: Vec2,
    radius: f64,
) -> Result<(Geom2D, Geom2D, Geom2D)> {
    if !radius.is_finite() || radius <= 0.0 {
        return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
    }
    let p = line_intersection(a, b, c, d)
        .ok_or_else(|| Reason::from_code(ReasonCode::GeomNoIntersection))?;
    let ua = norm(sub(a, p)).ok_or_else(|| Reason::from_code(ReasonCode::GeomDegenerate))?;
    let ub = norm(sub(c, p)).ok_or_else(|| Reason::from_code(ReasonCode::GeomDegenerate))?;
    let cos_t = dot(ua, ub).clamp(-1.0, 1.0);
    let theta = cos_t.acos();
    if theta <= 1e-6 || (std::f64::consts::PI - theta) <= 1e-6 {
        return Err(Reason::from_code(ReasonCode::EditFilletRadiusTooLarge));
    }
    let t = radius / (theta / 2.0).tan();
    let pa = add(p, mul(ua, t));
    let pb = add(p, mul(ub, t));
    if !pa.x.is_finite() || !pb.x.is_finite() {
        return Err(Reason::from_code(ReasonCode::EditFilletRadiusTooLarge));
    }
    let bis =
        norm(add(ua, ub)).ok_or_else(|| Reason::from_code(ReasonCode::EditFilletRadiusTooLarge))?;
    let h = radius / (theta / 2.0).sin();
    let center = add(p, mul(bis, h));
    let a0 = (pa.y - center.y).atan2(pa.x - center.x);
    let a1 = (pb.y - center.y).atan2(pb.x - center.x);
    Ok((
        Geom2D::Line { a, b: pa },
        Geom2D::Line { a: c, b: pb },
        Geom2D::Arc {
            c: center,
            r: radius,
            start_angle: a0,
            end_angle: a1,
            ccw: true,
        },
    ))
}

fn line_intersection(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Option<Vec2> {
    let r = sub(b, a);
    let s = sub(d, c);
    let den = r.x * s.y - r.y * s.x;
    if den.abs() <= 1e-12 {
        return None;
    }
    let t = ((c.x - a.x) * s.y - (c.y - a.y) * s.x) / den;
    Some(add(a, mul(r, t)))
}
