use crate::{
    util::{dist2, dot, lerp, sub},
    EpsilonPolicy, Geom2D, ProjectHit, Vec2,
};
use craftcad_serialize::{Reason, ReasonCode, Result};

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
        _ => Err(Reason::from_code(ReasonCode::GeomDegenerate)),
    }
}
