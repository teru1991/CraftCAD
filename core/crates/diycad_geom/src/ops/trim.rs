use crate::{ops::intersect::intersect, ops::project::project_point, EpsilonPolicy, Geom2D, Vec2};
use craftcad_serialize::{Reason, ReasonCode, Result};

fn line_param(a: Vec2, b: Vec2, p: Vec2) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let denom = dx * dx + dy * dy;
    if denom <= 0.0 {
        return 0.0;
    }
    ((p.x - a.x) * dx + (p.y - a.y) * dy) / denom
}

fn choose_candidate(
    candidates: &[Vec2],
    a: Vec2,
    b: Vec2,
    pick_point: Vec2,
    eps: &EpsilonPolicy,
    candidate_index: Option<usize>,
) -> Result<Vec2> {
    if candidates.is_empty() {
        return Err(Reason::from_code(ReasonCode::GeomTrimNoIntersection));
    }
    let t_pick = line_param(a, b, pick_point);
    let mut ranked: Vec<(usize, Vec2, f64, f64)> = candidates
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let t = line_param(a, b, *p);
            (i, *p, t, (t - t_pick).abs())
        })
        .collect();
    ranked.sort_by(|l, r| l.3.partial_cmp(&r.3).unwrap_or(std::cmp::Ordering::Equal));

    if let Some(idx) = candidate_index {
        if idx >= ranked.len() {
            let mut err = Reason::from_code(ReasonCode::EditTrimAmbiguousCandidate);
            err.debug.insert(
                "candidates".into(),
                serde_json::json!(ranked
                    .iter()
                    .map(|x| serde_json::json!({"index":x.0,"x":x.1.x,"y":x.1.y,"t":x.2}))
                    .collect::<Vec<_>>()),
            );
            err.debug
                .insert("invalid_candidate_index".into(), serde_json::json!(idx));
            return Err(err);
        }
        return Ok(ranked[idx].1);
    }

    if ranked.len() >= 2 && (ranked[0].3 - ranked[1].3).abs() <= eps.eq_dist {
        let mut err = Reason::from_code(ReasonCode::EditTrimAmbiguousCandidate);
        err.debug.insert(
            "candidates".into(),
            serde_json::json!(ranked
                .iter()
                .map(|x| serde_json::json!({"index":x.0,"x":x.1.x,"y":x.1.y,"t":x.2}))
                .collect::<Vec<_>>()),
        );
        return Err(err);
    }

    Ok(ranked[0].1)
}

pub fn trim_line_to_intersection(
    target_line: &Geom2D,
    cutter_geom: &Geom2D,
    pick_point: Vec2,
    eps: &EpsilonPolicy,
    candidate_index: Option<usize>,
) -> Result<Geom2D> {
    let (a, b) = match target_line {
        Geom2D::Line { a, b } => (*a, *b),
        _ => return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
    };
    let set = intersect(target_line, cutter_geom, eps).or_else(|r| {
        if r.code == "GEOM_NO_INTERSECTION" {
            Err(Reason::from_code(ReasonCode::GeomTrimNoIntersection))
        } else {
            Err(r)
        }
    })?;
    let p = choose_candidate(&set.points, a, b, pick_point, eps, candidate_index)?;
    let t_int = line_param(a, b, p);
    let t_pick = line_param(a, b, pick_point);
    if t_pick <= t_int {
        Ok(Geom2D::Line { a, b: p })
    } else {
        Ok(Geom2D::Line { a: p, b })
    }
}

pub fn trim_polyline_to_intersection(
    target: &Geom2D,
    cutter_geom: &Geom2D,
    pick_point: Vec2,
    eps: &EpsilonPolicy,
    candidate_index: Option<usize>,
) -> Result<Geom2D> {
    let (pts, closed) = match target {
        Geom2D::Polyline { pts, closed } => (pts.clone(), *closed),
        _ => return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
    };
    if closed || pts.len() < 2 {
        return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported));
    }

    let hit = project_point(target, pick_point, eps)?;
    let seg_count = pts.len() - 1;
    let u = hit.t_global * seg_count as f64;
    let i = (u.floor() as usize).min(seg_count - 1);

    let seg = Geom2D::Line {
        a: pts[i],
        b: pts[i + 1],
    };
    let trimmed = trim_line_to_intersection(&seg, cutter_geom, pick_point, eps, candidate_index)?;
    let (na, nb) = match trimmed {
        Geom2D::Line { a, b } => (a, b),
        _ => unreachable!(),
    };

    let mut out = pts;
    out[i] = na;
    out[i + 1] = nb;
    Ok(Geom2D::Polyline {
        pts: out,
        closed: false,
    })
}
