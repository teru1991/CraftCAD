use crate::model::{Entity, InternalModel, Pt, Segment2D};

fn sanitize_pt(pt: &mut Pt) -> bool {
    let mut changed = false;
    if !pt.x.is_finite() {
        pt.x = 0.0;
        changed = true;
    }
    if !pt.y.is_finite() {
        pt.y = 0.0;
        changed = true;
    }
    changed
}

fn round_f64(v: f64, step: f64) -> f64 {
    if step <= 0.0 {
        return v;
    }
    (v / step).round() * step
}

fn distance(a: Pt, b: Pt) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn path_endpoints(path: &[Segment2D]) -> Option<(Pt, Pt)> {
    let first = path.first()?;
    let last = path.last()?;
    let first_pt = match first {
        Segment2D::Line { to } | Segment2D::Arc { to, .. } | Segment2D::CubicBezier { to, .. } => {
            *to
        }
    };
    let last_pt = match last {
        Segment2D::Line { to } | Segment2D::Arc { to, .. } | Segment2D::CubicBezier { to, .. } => {
            *to
        }
    };
    Some((first_pt, last_pt))
}

fn entity_sort_key(entity: &Entity) -> (String, u8, String) {
    match entity {
        Entity::Path(path) => (path.layer.clone(), 0, path.id.clone()),
        Entity::Text(text) => (text.layer.clone(), 1, text.id.clone()),
    }
}

pub fn normalize_model(m: &mut InternalModel, eps: f64) {
    for entity in &mut m.entities {
        match entity {
            Entity::Path(path) => {
                for segment in &mut path.path.segments {
                    match segment {
                        Segment2D::Line { to } => {
                            sanitize_pt(to);
                            to.x = round_f64(to.x, eps);
                            to.y = round_f64(to.y, eps);
                        }
                        Segment2D::Arc { to, center, .. } => {
                            sanitize_pt(to);
                            sanitize_pt(center);
                            to.x = round_f64(to.x, eps);
                            to.y = round_f64(to.y, eps);
                            center.x = round_f64(center.x, eps);
                            center.y = round_f64(center.y, eps);
                        }
                        Segment2D::CubicBezier { c1, c2, to } => {
                            sanitize_pt(c1);
                            sanitize_pt(c2);
                            sanitize_pt(to);
                            c1.x = round_f64(c1.x, eps);
                            c1.y = round_f64(c1.y, eps);
                            c2.x = round_f64(c2.x, eps);
                            c2.y = round_f64(c2.y, eps);
                            to.x = round_f64(to.x, eps);
                            to.y = round_f64(to.y, eps);
                        }
                    }
                }

                if let Some((start, end)) = path_endpoints(&path.path.segments) {
                    if distance(start, end) <= eps {
                        path.path.closed = true;
                    }
                }
            }
            Entity::Text(text) => {
                sanitize_pt(&mut text.anchor);
                text.anchor.x = round_f64(text.anchor.x, eps);
                text.anchor.y = round_f64(text.anchor.y, eps);
            }
        }
    }

    m.entities.sort_by_key(entity_sort_key);
}
