use crate::{math::approx_eq, Pt};

pub fn line_line(a1: Pt, a2: Pt, b1: Pt, b2: Pt, eps: f64) -> Option<Pt> {
    let d = (a1.x - a2.x) * (b1.y - b2.y) - (a1.y - a2.y) * (b1.x - b2.x);
    if approx_eq(d, 0.0, eps) {
        return None;
    }
    let pre = a1.x * a2.y - a1.y * a2.x;
    let post = b1.x * b2.y - b1.y * b2.x;
    Some(Pt {
        x: (pre * (b1.x - b2.x) - (a1.x - a2.x) * post) / d,
        y: (pre * (b1.y - b2.y) - (a1.y - a2.y) * post) / d,
    })
}
