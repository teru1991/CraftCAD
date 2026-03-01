use craftcad_serialize::Vec2;

pub fn signed_area(pts: &[Vec2]) -> f64 {
    if pts.len() < 3 {
        return 0.0;
    }
    let mut a = 0.0;
    for i in 0..pts.len() {
        let p = &pts[i];
        let q = &pts[(i + 1) % pts.len()];
        a += p.x * q.y - q.x * p.y;
    }
    0.5 * a
}

pub fn ensure_ccw(pts: &mut Vec<Vec2>) {
    if signed_area(pts) < 0.0 {
        pts.reverse();
    }
}

pub fn ensure_cw(pts: &mut Vec<Vec2>) {
    if signed_area(pts) > 0.0 {
        pts.reverse();
    }
}
