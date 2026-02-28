use crate::{
    ops::project::project_point, util::lerp, EpsilonPolicy, Geom2D, SplitBy, SplitResult, Vec2,
};
use craftcad_serialize::{Reason, ReasonCode, Result};

pub fn split_at(g: &Geom2D, by: SplitBy, eps: &EpsilonPolicy) -> Result<SplitResult> {
    match g {
        Geom2D::Line { a, b } => {
            if (a.x - b.x).powi(2) + (a.y - b.y).powi(2) <= eps.eq_dist * eps.eq_dist {
                return Err(Reason::from_code(ReasonCode::GeomDegenerate));
            }
            let t = match by {
                SplitBy::T(t) => t,
                SplitBy::Point(p) => {
                    let hit = project_point(g, p, eps)?;
                    if hit.dist > eps.snap_dist {
                        return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
                    }
                    hit.t_global
                }
            };
            if t <= eps.eq_dist || t >= 1.0 - eps.eq_dist {
                return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
            }
            let m = lerp(*a, *b, t);
            Ok(SplitResult {
                left: Geom2D::Line { a: *a, b: m },
                right: Geom2D::Line { a: m, b: *b },
                split_point: m,
            })
        }
        Geom2D::Polyline { pts, closed } => {
            if *closed {
                let mut r = Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom);
                r.debug.insert(
                    "note".into(),
                    serde_json::json!("closed_polyline_split_not_supported_v1"),
                );
                return Err(r);
            }
            let seg_count = pts.len().saturating_sub(1);
            if seg_count == 0 {
                return Err(Reason::from_code(ReasonCode::GeomDegenerate));
            }
            let tg = match by {
                SplitBy::T(t) => t,
                SplitBy::Point(p) => {
                    let hit = project_point(g, p, eps)?;
                    if hit.dist > eps.snap_dist {
                        return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
                    }
                    hit.t_global
                }
            };
            if !(0.0..=1.0).contains(&tg) {
                return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
            }
            let u = tg * seg_count as f64;
            let i = (u.floor() as usize).min(seg_count - 1);
            let t_local = u - i as f64;
            if t_local <= eps.eq_dist || t_local >= 1.0 - eps.eq_dist {
                return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
            }
            let split = lerp(pts[i], pts[i + 1], t_local);

            let mut left_pts: Vec<Vec2> = pts[0..=i].to_vec();
            left_pts.push(split);
            let mut right_pts: Vec<Vec2> = vec![split];
            right_pts.extend_from_slice(&pts[i + 1..]);
            if left_pts.len() < 2 || right_pts.len() < 2 {
                return Err(Reason::from_code(ReasonCode::GeomDegenerate));
            }

            Ok(SplitResult {
                left: Geom2D::Polyline {
                    pts: left_pts,
                    closed: false,
                },
                right: Geom2D::Polyline {
                    pts: right_pts,
                    closed: false,
                },
                split_point: split,
            })
        }
        _ => Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom)),
    }
}
