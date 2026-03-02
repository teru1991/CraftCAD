use drawing_model::{GeometryRef, RefKind};

#[derive(Debug, Clone)]
pub enum ResolvedGeom {
    Point { p: (f64, f64) },
    Segment { a: (f64, f64), b: (f64, f64) },
    Circle { c: (f64, f64), r_mm: f64 },
}

pub trait RefResolver {
    fn resolve(&self, r: &GeometryRef) -> Option<ResolvedGeom>;
}

pub struct SampleBoardResolver;

impl RefResolver for SampleBoardResolver {
    fn resolve(&self, r: &GeometryRef) -> Option<ResolvedGeom> {
        match (r.kind.clone(), r.stable_id.as_str()) {
            (RefKind::Point, "P_ORIGIN") => Some(ResolvedGeom::Point { p: (30.0, 30.0) }),
            (RefKind::Point, "P_BOARD_A") => Some(ResolvedGeom::Point { p: (30.0, 30.0) }),
            (RefKind::Point, "P_BOARD_B") => Some(ResolvedGeom::Point { p: (180.0, 30.0) }),
            (RefKind::Point, "P_BOARD_C") => Some(ResolvedGeom::Point { p: (180.0, 130.0) }),
            (RefKind::Point, "P_BOARD_D") => Some(ResolvedGeom::Point { p: (30.0, 130.0) }),
            (RefKind::Circle, "HOLE_1") => Some(ResolvedGeom::Circle {
                c: (90.0, 80.0),
                r_mm: 5.0,
            }),
            (RefKind::Segment, "SEG_TOP") => Some(ResolvedGeom::Segment {
                a: (30.0, 30.0),
                b: (180.0, 30.0),
            }),
            (RefKind::Segment, "SEG_LEFT") => Some(ResolvedGeom::Segment {
                a: (30.0, 30.0),
                b: (30.0, 130.0),
            }),
            _ => None,
        }
    }
}
