use crate::ssot::StylePreset;
use crate::units::fmt_diameter;

pub fn hole_text(style: &StylePreset, dia_mm: f64, count: Option<u32>) -> String {
    let mut s = fmt_diameter(style, dia_mm, None);
    if let Some(n) = count {
        s.push_str(&format!(" x{n}"));
    }
    s
}
