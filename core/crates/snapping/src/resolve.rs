use crate::types::{SnapPoint, SnapResult};

pub struct SnapPolicy {
    pub max_dist: f64,
    pub angle_step_deg: f64,
    pub prefer: Vec<i32>,
}

pub fn resolve(
    mut cands: Vec<SnapPoint>,
    cursor: craftcad_geom2d::Pt,
    policy: &SnapPolicy,
) -> SnapResult {
    cands.retain(|c| craftcad_geom2d::dist(c.pt, cursor) <= policy.max_dist);
    cands.sort_by(|a, b| {
        let score_ord = b.score.cmp(&a.score);
        if score_ord != std::cmp::Ordering::Equal {
            return score_ord;
        }
        let da = craftcad_geom2d::dist(a.pt, cursor);
        let db = craftcad_geom2d::dist(b.pt, cursor);
        let dist_ord = da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal);
        if dist_ord != std::cmp::Ordering::Equal {
            return dist_ord;
        }
        let x_ord =
            a.pt.x
                .partial_cmp(&b.pt.x)
                .unwrap_or(std::cmp::Ordering::Equal);
        if x_ord != std::cmp::Ordering::Equal {
            return x_ord;
        }
        let y_ord =
            a.pt.y
                .partial_cmp(&b.pt.y)
                .unwrap_or(std::cmp::Ordering::Equal);
        if y_ord != std::cmp::Ordering::Equal {
            return y_ord;
        }
        a.kind.cmp(&b.kind)
    });
    let snapped = cands.first().copied();
    SnapResult {
        snapped,
        candidates: cands,
    }
}
