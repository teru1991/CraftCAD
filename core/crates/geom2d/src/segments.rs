use crate::Pt;

#[derive(Clone, Debug)]
pub enum Segment {
    Line { a: Pt, b: Pt },
    Arc { c: Pt, r: f64, a0: f64, a1: f64 },
    Bezier { p0: Pt, p1: Pt, p2: Pt, p3: Pt },
}
