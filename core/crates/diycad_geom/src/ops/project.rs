use crate::{
    util::{dist2, dot, lerp, sub},
    EpsilonPolicy, Geom2D, ProjectHit, Vec2,
};
use craftcad_serialize::{Reason, ReasonCode, Result};

fn normalize_angle(mut a: f64) -> f64 {
    while a <= -std::f64::consts::PI {
        a += 2.0 * std::f64::consts::PI;
    }
    while a > std::f64::consts::PI {
        a -= 2.0 * std::f64::consts::PI;
    }
    a
}

fn arc_sweep(start: f64, end: f64, ccw: bool) -> f64 {
    let s = normalize_angle(start);
    let e = normalize_angle(end);
    if ccw {
        let mut d = e - s;
        if d < 0.0 {
            d += 2.0 * std::f64::consts::PI;
        }
        d
    } else {
        let mut d = s - e;
        if d < 0.0 {
            d += 2.0 * std::f64::consts::PI;
        }
        d
    }
}

fn in_arc_range(theta: f64, start: f64, end: f64, ccw: bool, eps: f64) -> bool {
    let t = normalize_angle(theta);
    let s = normalize_angle(start);
    let sweep = arc_sweep(start, end, ccw);
    if ccw {
        let mut u = t - s;
        if u < 0.0 {
            u += 2.0 * std::f64::consts::PI;
        }
        u >= -eps && u <= sweep + eps
    } else {
        let mut u = s - t;
        if u < 0.0 {
            u += 2.0 * std::f64::consts::PI;
        }
        u >= -eps && u <= sweep + eps
    }
}

fn angle_to_t(theta: f64, start: f64, end: f64, ccw: bool) -> f64 {
    let s = normalize_angle(start);
    let t = normalize_angle(theta);
    let sweep = arc_sweep(start, end, ccw).max(1e-12);
    if ccw {
        let mut u = t - s;
        if u < 0.0 {
            u += 2.0 * std::f64::consts::PI;
        }
        (u / sweep).clamp(0.0, 1.0)
    } else {
        let mut u = s - t;
        if u < 0.0 {
            u += 2.0 * std::f64::consts::PI;
        }
        (u / sweep).clamp(0.0, 1.0)
    }
}

fn project_line(a: Vec2, b: Vec2, p: Vec2) -> Result<(Vec2, f64, f64)> {
    let ab = sub(b, a);
    let ap = sub(p, a);
    let denom = dot(ab, ab);
    if denom <= 0.0 {
        return Err(Reason::from_code(ReasonCode::GeomDegenerate));
    }
    let t = (dot(ap, ab) / denom).clamp(0.0, 1.0);
    let q = lerp(a, b, t);
    Ok((q, t, dist2(p, q).sqrt()))
}

pub fn project_point(g: &Geom2D, p: Vec2, _eps: &EpsilonPolicy) -> Result<ProjectHit> {
    match g {
        Geom2D::Line { a, b } => {
            let (q, t, d) = project_line(*a, *b, p)?;
            Ok(ProjectHit {
                point: q,
                t_global: t,
                dist: d,
            })
        }
        Geom2D::Circle { c, r } => {
            if !r.is_finite() || *r <= 0.0 {
                return Err(Reason::from_code(ReasonCode::GeomCircleRadiusInvalid));
            }
            let vx = p.x - c.x;
            let vy = p.y - c.y;
            let len2 = vx * vx + vy * vy;
            let theta = if len2 <= f64::EPSILON {
                0.0
            } else {
                vy.atan2(vx)
            };
            let q = Vec2 {
                x: c.x + r * theta.cos(),
                y: c.y + r * theta.sin(),
            };
            Ok(ProjectHit {
                point: q,
                t_global: (normalize_angle(theta) + std::f64::consts::PI)
                    / (2.0 * std::f64::consts::PI),
                dist: dist2(p, q).sqrt(),
            })
        }
        Geom2D::Arc {
            c,
            r,
            start_angle,
            end_angle,
            ccw,
        } => {
            if !r.is_finite() || *r <= 0.0 {
                return Err(Reason::from_code(ReasonCode::GeomCircleRadiusInvalid));
            }
            if !start_angle.is_finite() || !end_angle.is_finite() {
                return Err(Reason::from_code(ReasonCode::GeomArcRangeInvalid));
            }

            let theta = (p.y - c.y).atan2(p.x - c.x);
            let cand_theta = normalize_angle(theta);
            let start = normalize_angle(*start_angle);
            let end = normalize_angle(*end_angle);

            let q_on_circle = Vec2 {
                x: c.x + r * cand_theta.cos(),
                y: c.y + r * cand_theta.sin(),
            };
            let q_start = Vec2 {
                x: c.x + r * start.cos(),
                y: c.y + r * start.sin(),
            };
            let q_end = Vec2 {
                x: c.x + r * end.cos(),
                y: c.y + r * end.sin(),
            };

            let in_range = in_arc_range(cand_theta, *start_angle, *end_angle, *ccw, 1e-12);

            let (q, t_global) = if in_range {
                (
                    q_on_circle,
                    angle_to_t(cand_theta, *start_angle, *end_angle, *ccw),
                )
            } else {
                let ds = dist2(p, q_start);
                let de = dist2(p, q_end);
                if ds <= de {
                    (q_start, 0.0)
                } else {
                    (q_end, 1.0)
                }
            };

            Ok(ProjectHit {
                point: q,
                t_global,
                dist: dist2(p, q).sqrt(),
            })
        }
        // Polyline global parameterization:
        // seg_count = closed ? pts.len() : pts.len()-1
        // global t = (i + t_local) / seg_count
        Geom2D::Polyline { pts, closed } => {
            let seg_count = if *closed {
                pts.len()
            } else {
                pts.len().saturating_sub(1)
            };
            if seg_count == 0 {
                return Err(Reason::from_code(ReasonCode::GeomDegenerate));
            }
            let mut best: Option<ProjectHit> = None;
            for i in 0..seg_count {
                let a = pts[i];
                let b = if i + 1 < pts.len() {
                    pts[i + 1]
                } else {
                    pts[0]
                };
                let (q, t_local, d) = project_line(a, b, p)?;
                let t_global = (i as f64 + t_local) / seg_count as f64;
                let hit = ProjectHit {
                    point: q,
                    t_global,
                    dist: d,
                };
                if best.as_ref().map(|h| d < h.dist).unwrap_or(true) {
                    best = Some(hit);
                }
            }
            best.ok_or_else(|| Reason::from_code(ReasonCode::GeomDegenerate))
        }
    }
}
