use craftcad_geom2d::Pt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SnapKind {
    Endpoint,
    Midpoint,
    Intersection,
    Center,
    Grid,
    Angle,
}

#[derive(Clone, Copy, Debug)]
pub struct SnapPoint {
    pub kind: SnapKind,
    pub pt: Pt,
    pub score: i32,
}

#[derive(Clone, Debug)]
pub struct SnapResult {
    pub snapped: Option<SnapPoint>,
    pub candidates: Vec<SnapPoint>,
}
