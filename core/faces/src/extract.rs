use crate::winding::{ensure_ccw, ensure_cw, signed_area};
use crate::{Face, FaceSet, Polygon};
use craftcad_serialize::{Geom2D, Reason, ReasonCode, Result, Vec2};
use diycad_geom::EpsilonPolicy;

#[derive(Clone)]
struct LoopInfo {
    pts: Vec<Vec2>,
    area_abs: f64,
    parent: Option<usize>,
    depth: usize,
}

fn finite(p: &Vec2) -> bool {
    p.x.is_finite() && p.y.is_finite()
}

fn sub(a: &Vec2, b: &Vec2) -> Vec2 {
    Vec2 {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn cross(a: &Vec2, b: &Vec2) -> f64 {
    a.x * b.y - a.y * b.x
}

fn orient(a: &Vec2, b: &Vec2, c: &Vec2) -> f64 {
    cross(&sub(b, a), &sub(c, a))
}

fn on_segment(a: &Vec2, b: &Vec2, p: &Vec2, eps: &EpsilonPolicy) -> bool {
    let c = orient(a, b, p).abs();
    if c > eps.intersect_tol {
        return false;
    }
    let minx = a.x.min(b.x) - eps.eq_dist;
    let maxx = a.x.max(b.x) + eps.eq_dist;
    let miny = a.y.min(b.y) - eps.eq_dist;
    let maxy = a.y.max(b.y) + eps.eq_dist;
    p.x >= minx && p.x <= maxx && p.y >= miny && p.y <= maxy
}

fn seg_intersect(a1: &Vec2, a2: &Vec2, b1: &Vec2, b2: &Vec2, eps: &EpsilonPolicy) -> bool {
    let o1 = orient(a1, a2, b1);
    let o2 = orient(a1, a2, b2);
    let o3 = orient(b1, b2, a1);
    let o4 = orient(b1, b2, a2);

    if ((o1 > eps.intersect_tol && o2 < -eps.intersect_tol)
        || (o1 < -eps.intersect_tol && o2 > eps.intersect_tol))
        && ((o3 > eps.intersect_tol && o4 < -eps.intersect_tol)
            || (o3 < -eps.intersect_tol && o4 > eps.intersect_tol))
    {
        return true;
    }

    on_segment(a1, a2, b1, eps)
        || on_segment(a1, a2, b2, eps)
        || on_segment(b1, b2, a1, eps)
        || on_segment(b1, b2, a2, eps)
}

fn self_intersects(pts: &[Vec2], eps: &EpsilonPolicy) -> Option<(usize, usize)> {
    let n = pts.len();
    for i in 0..n {
        let i2 = (i + 1) % n;
        for j in (i + 1)..n {
            let j2 = (j + 1) % n;
            if i == j || i2 == j || j2 == i {
                continue;
            }
            if i == 0 && j2 == 0 {
                continue;
            }
            if seg_intersect(&pts[i], &pts[i2], &pts[j], &pts[j2], eps) {
                return Some((i, j));
            }
        }
    }
    None
}

#[derive(PartialEq, Eq)]
enum Pip {
    In,
    Out,
    Boundary,
}

fn point_in_poly(p: &Vec2, pts: &[Vec2], eps: &EpsilonPolicy) -> Pip {
    let mut inside = false;
    for i in 0..pts.len() {
        let a = &pts[i];
        let b = &pts[(i + 1) % pts.len()];
        if on_segment(a, b, p, eps) {
            return Pip::Boundary;
        }
        if (a.y > p.y) != (b.y > p.y) {
            let denom = b.y - a.y;
            if denom.abs() <= eps.eq_dist {
                continue;
            }
            let x_at_y = (b.x - a.x) * (p.y - a.y) / denom + a.x;
            if p.x < x_at_y {
                inside = !inside;
            }
        }
    }
    if inside {
        Pip::In
    } else {
        Pip::Out
    }
}

pub fn extract_faces(polylines: &[Geom2D], eps: &EpsilonPolicy) -> Result<FaceSet> {
    let mut loops: Vec<LoopInfo> = Vec::new();
    for g in polylines {
        let pts0 = match g {
            Geom2D::Polyline { pts, closed: true } => pts,
            _ => continue,
        };
        let mut pts = pts0.clone();
        if pts.len() >= 2 {
            let f = &pts[0];
            let l = pts.last().expect("last");
            if (f.x - l.x).abs() <= eps.eq_dist && (f.y - l.y).abs() <= eps.eq_dist {
                pts.pop();
            }
        }
        if pts.len() < 3 {
            return Err(Reason::from_code(ReasonCode::FaceNoClosedLoop));
        }
        if pts.iter().any(|p| !finite(p)) {
            return Err(Reason::from_code(ReasonCode::GeomInvalidNumeric));
        }
        if let Some((i, j)) = self_intersects(&pts, eps) {
            let mut r = Reason::from_code(ReasonCode::FaceSelfIntersection);
            r.debug.insert("seg_i".into(), serde_json::json!(i));
            r.debug.insert("seg_j".into(), serde_json::json!(j));
            return Err(r);
        }
        loops.push(LoopInfo {
            area_abs: signed_area(&pts).abs(),
            pts,
            parent: None,
            depth: 0,
        });
    }

    if loops.is_empty() {
        return Err(Reason::from_code(ReasonCode::FaceNoClosedLoop));
    }

    for i in 0..loops.len() {
        let sample = loops[i].pts[0].clone();
        let mut parents: Vec<usize> = Vec::new();
        for j in 0..loops.len() {
            if i == j {
                continue;
            }
            match point_in_poly(&sample, &loops[j].pts, eps) {
                Pip::Boundary => {
                    let mut r = Reason::from_code(ReasonCode::FaceAmbiguousLoop);
                    r.debug.insert("loop_i".into(), serde_json::json!(i));
                    r.debug.insert("loop_j".into(), serde_json::json!(j));
                    return Err(r);
                }
                Pip::In => parents.push(j),
                Pip::Out => {}
            }
        }
        parents.sort_by(|a, b| loops[*a].area_abs.partial_cmp(&loops[*b].area_abs).unwrap());
        loops[i].parent = parents.first().copied();
    }

    fn depth(i: usize, loops: &mut [LoopInfo]) -> usize {
        if loops[i].depth != 0 || loops[i].parent.is_none() {
            return loops[i].depth;
        }
        let p = loops[i].parent.unwrap();
        loops[i].depth = depth(p, loops) + 1;
        loops[i].depth
    }
    for i in 0..loops.len() {
        let _ = depth(i, &mut loops);
    }

    let mut faces = Vec::new();
    for i in 0..loops.len() {
        if loops[i].depth % 2 != 0 {
            continue;
        }
        let mut outer_pts = loops[i].pts.clone();
        ensure_ccw(&mut outer_pts);
        let mut holes = Vec::new();
        for j in 0..loops.len() {
            if loops[j].parent == Some(i) && loops[j].depth % 2 == 1 {
                let mut h = loops[j].pts.clone();
                ensure_cw(&mut h);
                holes.push(Polygon { pts: h });
            }
        }
        faces.push(Face {
            outer: Polygon { pts: outer_pts },
            holes,
        });
    }

    faces.sort_by(|a, b| {
        signed_area(&b.outer.pts)
            .abs()
            .partial_cmp(&signed_area(&a.outer.pts).abs())
            .unwrap()
    });

    Ok(FaceSet { faces })
}
