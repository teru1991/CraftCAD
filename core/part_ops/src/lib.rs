use craftcad_faces::Face;
use craftcad_serialize::{Part, Polygon2D, Reason, ReasonCode, Result};

pub fn create_part_from_face(face: &Face, mut part: Part) -> Result<Part> {
    if face.outer.pts.len() < 3
        || face
            .outer
            .pts
            .iter()
            .any(|p| !p.x.is_finite() || !p.y.is_finite())
    {
        return Err(Reason::from_code(ReasonCode::PartInvalidOutline));
    }
    for h in &face.holes {
        if h.pts.len() < 3 || h.pts.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return Err(Reason::from_code(ReasonCode::PartInvalidOutline));
        }
    }
    part.outline = Polygon2D {
        outer: face.outer.pts.clone(),
        holes: face.holes.iter().map(|h| h.pts.clone()).collect(),
    };
    Ok(part)
}
