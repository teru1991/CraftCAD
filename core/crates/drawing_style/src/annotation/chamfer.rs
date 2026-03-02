use crate::ssot::StylePreset;
use crate::units::{fmt_chamfer_c, fmt_radius};

pub fn chamfer_text(style: &StylePreset, ty: &str, value_mm: f64) -> String {
    match ty {
        "R" => fmt_radius(style, value_mm, None),
        _ => fmt_chamfer_c(style, value_mm, None),
    }
}
