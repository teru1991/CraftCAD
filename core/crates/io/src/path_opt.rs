use crate::model::*;
use crate::reasons::{AppError, ReasonCode};
use std::cmp::Ordering;

fn near(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

fn near_pt(a: Point2D, b: Point2D, eps: f64) -> bool {
    near(a.x, b.x, eps) && near(a.y, b.y, eps)
}

fn bbox_of_path(p: &PathEntity) -> (f64, f64, f64, f64) {
    let mut minx = f64::INFINITY;
    let mut miny = f64::INFINITY;
    let mut maxx = f64::NEG_INFINITY;
    let mut maxy = f64::NEG_INFINITY;

    for s in &p.segments {
        match s {
            Segment2D::Line { a, b } => {
                minx = minx.min(a.x).min(b.x);
                miny = miny.min(a.y).min(b.y);
                maxx = maxx.max(a.x).max(b.x);
                maxy = maxy.max(a.y).max(b.y);
            }
            Segment2D::CubicBezier { a, c1, c2, b } => {
                for pt in [*a, *c1, *c2, *b] {
                    minx = minx.min(pt.x);
                    miny = miny.min(pt.y);
                    maxx = maxx.max(pt.x);
                    maxy = maxy.max(pt.y);
                }
            }
            Segment2D::Arc { center, radius, .. } | Segment2D::Circle { center, radius } => {
                minx = minx.min(center.x - radius);
                miny = miny.min(center.y - radius);
                maxx = maxx.max(center.x + radius);
                maxy = maxy.max(center.y + radius);
            }
        }
    }

    if !minx.is_finite() {
        (0.0, 0.0, 0.0, 0.0)
    } else {
        (minx, miny, maxx, maxy)
    }
}

fn cmp_f64(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

fn path_sort_key(p: &PathEntity) -> (String, String, String, f64, f64, f64, f64, String) {
    let (minx, miny, maxx, maxy) = bbox_of_path(p);
    (
        p.stroke.layer.clone(),
        p.stroke.linetype.clone(),
        "Path".to_string(),
        minx,
        miny,
        maxx,
        maxy,
        p.id.clone(),
    )
}

fn text_sort_key(t: &TextEntity) -> (String, String, String, f64, f64, String) {
    (
        t.layer.clone(),
        "CONTINUOUS".to_string(),
        "Text".to_string(),
        t.pos.x,
        t.pos.y,
        t.id.clone(),
    )
}

pub fn postprocess_model(model: &mut InternalModel, eps: f64, warnings: &mut Vec<AppError>) {
    for ent in &mut model.entities {
        if let Entity::Path(p) = ent {
            let mut out: Vec<Segment2D> = Vec::new();
            for s in &p.segments {
                if let Some(Segment2D::Line { a: a1, b: b1 }) = out.last().cloned() {
                    if let Segment2D::Line { a: a2, b: b2 } = s {
                        if near_pt(b1, *a2, eps) {
                            let area =
                                (b1.x - a1.x) * (b2.y - a1.y) - (b1.y - a1.y) * (b2.x - a1.x);
                            if area.abs() <= eps {
                                out.pop();
                                out.push(Segment2D::Line { a: a1, b: *b2 });
                                continue;
                            }
                        }
                    }
                }
                if let Some(prev) = out.last() {
                    if prev == s {
                        continue;
                    }
                }
                out.push(s.clone());
            }

            if out.len() != p.segments.len() {
                warnings.push(
                    AppError::new(
                        ReasonCode::IO_PATH_JOIN_APPLIED,
                        "path optimization applied",
                    )
                    .with_context("path_id", p.id.clone())
                    .with_context("before", p.segments.len().to_string())
                    .with_context("after", out.len().to_string()),
                );
            }
            p.segments = out;
        }
    }

    model.entities.sort_by(|a, b| match (a, b) {
        (Entity::Path(pa), Entity::Path(pb)) => {
            let ka = path_sort_key(pa);
            let kb = path_sort_key(pb);
            ka.0.cmp(&kb.0)
                .then_with(|| ka.1.cmp(&kb.1))
                .then_with(|| ka.2.cmp(&kb.2))
                .then_with(|| cmp_f64(ka.3, kb.3))
                .then_with(|| cmp_f64(ka.4, kb.4))
                .then_with(|| cmp_f64(ka.5, kb.5))
                .then_with(|| cmp_f64(ka.6, kb.6))
                .then_with(|| ka.7.cmp(&kb.7))
        }
        (Entity::Text(ta), Entity::Text(tb)) => {
            let ka = text_sort_key(ta);
            let kb = text_sort_key(tb);
            ka.0.cmp(&kb.0)
                .then_with(|| ka.1.cmp(&kb.1))
                .then_with(|| ka.2.cmp(&kb.2))
                .then_with(|| cmp_f64(ka.3, kb.3))
                .then_with(|| cmp_f64(ka.4, kb.4))
                .then_with(|| ka.5.cmp(&kb.5))
        }
        (Entity::Path(_), Entity::Text(_)) => Ordering::Less,
        (Entity::Text(_), Entity::Path(_)) => Ordering::Greater,
    });
}
