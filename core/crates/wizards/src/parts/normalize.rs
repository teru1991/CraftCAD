use crate::parts::model::*;
use std::cmp::Ordering;

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

pub fn normalize(mut d: PartsDraft) -> PartsDraft {
    for p in &mut d.parts {
        match &mut p.outline {
            Outline2D::Rect { w_mm, h_mm } => {
                *w_mm = round3(*w_mm);
                *h_mm = round3(*h_mm);
            }
        }
        for f in &mut p.features {
            match f {
                Feature2D::HoleCircle {
                    cx_mm,
                    cy_mm,
                    diameter_mm,
                } => {
                    *cx_mm = round3(*cx_mm);
                    *cy_mm = round3(*cy_mm);
                    *diameter_mm = round3(*diameter_mm);
                }
                Feature2D::StitchHole {
                    x_mm,
                    y_mm,
                    diameter_mm,
                } => {
                    *x_mm = round3(*x_mm);
                    *y_mm = round3(*y_mm);
                    *diameter_mm = round3(*diameter_mm);
                }
            }
        }
        p.features.sort_by_key(feature_key);
        p.tags.sort();
        p.tags.dedup();
    }

    d.parts.sort_by(|a, b| match a.part_id.cmp(&b.part_id) {
        Ordering::Equal => a.name.cmp(&b.name),
        o => o,
    });

    d.annotations.sort_by(|a, b| {
        serde_json::to_string(a)
            .unwrap_or_default()
            .cmp(&serde_json::to_string(b).unwrap_or_default())
    });

    d
}

fn feature_key(f: &Feature2D) -> (u8, i64, i64, i64) {
    match *f {
        Feature2D::HoleCircle {
            cx_mm,
            cy_mm,
            diameter_mm,
        } => (
            0,
            (cx_mm * 1000.0).round() as i64,
            (cy_mm * 1000.0).round() as i64,
            (diameter_mm * 1000.0).round() as i64,
        ),
        Feature2D::StitchHole {
            x_mm,
            y_mm,
            diameter_mm,
        } => (
            1,
            (x_mm * 1000.0).round() as i64,
            (y_mm * 1000.0).round() as i64,
            (diameter_mm * 1000.0).round() as i64,
        ),
    }
}
