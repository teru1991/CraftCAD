mod extract;
pub mod winding;

use craftcad_serialize::Vec2;
use serde::{Deserialize, Serialize};

pub use extract::extract_faces;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polygon {
    pub pts: Vec<Vec2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Face {
    pub outer: Polygon,
    pub holes: Vec<Polygon>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceSet {
    pub faces: Vec<Face>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use craftcad_serialize::{Geom2D, Vec2};
    use diycad_geom::EpsilonPolicy;

    #[test]
    fn rectangle_one_face() {
        let g = Geom2D::Polyline {
            pts: vec![
                Vec2 { x: 0.0, y: 0.0 },
                Vec2 { x: 10.0, y: 0.0 },
                Vec2 { x: 10.0, y: 5.0 },
                Vec2 { x: 0.0, y: 5.0 },
            ],
            closed: true,
        };
        let fs = extract_faces(&[g], &EpsilonPolicy::default()).expect("ok");
        assert_eq!(fs.faces.len(), 1);
        assert_eq!(fs.faces[0].holes.len(), 0);
    }

    #[test]
    fn rectangle_with_hole() {
        let o = Geom2D::Polyline {
            pts: vec![
                Vec2 { x: 0.0, y: 0.0 },
                Vec2 { x: 10.0, y: 0.0 },
                Vec2 { x: 10.0, y: 10.0 },
                Vec2 { x: 0.0, y: 10.0 },
            ],
            closed: true,
        };
        let i = Geom2D::Polyline {
            pts: vec![
                Vec2 { x: 3.0, y: 3.0 },
                Vec2 { x: 7.0, y: 3.0 },
                Vec2 { x: 7.0, y: 7.0 },
                Vec2 { x: 3.0, y: 7.0 },
            ],
            closed: true,
        };
        let fs = extract_faces(&[o, i], &EpsilonPolicy::default()).expect("ok");
        assert_eq!(fs.faces.len(), 1);
        assert_eq!(fs.faces[0].holes.len(), 1);
    }

    #[test]
    fn self_intersection_errors() {
        let bow = Geom2D::Polyline {
            pts: vec![
                Vec2 { x: 0.0, y: 0.0 },
                Vec2 { x: 2.0, y: 2.0 },
                Vec2 { x: 0.0, y: 2.0 },
                Vec2 { x: 2.0, y: 0.0 },
            ],
            closed: true,
        };
        let err = extract_faces(&[bow], &EpsilonPolicy::default()).expect_err("err");
        assert_eq!(err.code, "FACE_SELF_INTERSECTION");
    }
}
