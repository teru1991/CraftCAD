use crate::{dist, Pt};

pub fn point_segment_distance(p: Pt, a: Pt, b: Pt) -> f64 {
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let ab2 = abx * abx + aby * aby;
    if ab2 <= f64::EPSILON {
        return dist(p, a);
    }
    let apx = p.x - a.x;
    let apy = p.y - a.y;
    let t = ((apx * abx + apy * aby) / ab2).clamp(0.0, 1.0);
    let q = Pt {
        x: a.x + t * abx,
        y: a.y + t * aby,
    };
    dist(p, q)
}
