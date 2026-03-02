use crate::ssot::StylePreset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayUnit {
    Mm,
    Inch,
}

pub fn display_unit(style: &StylePreset) -> DisplayUnit {
    match style.units.display.as_str() {
        "inch" => DisplayUnit::Inch,
        _ => DisplayUnit::Mm,
    }
}

pub fn mm_to_inch(mm: f64) -> f64 {
    mm / 25.4
}

pub fn round_step(v: f64, step: f64) -> f64 {
    if step <= 0.0 {
        return v;
    }
    (v / step).round() * step
}

pub fn fmt_length(style: &StylePreset, mm: f64, precision_override: Option<u8>) -> String {
    let unit = display_unit(style);
    let step = style.rounding.length_step;
    let prec = precision_override.unwrap_or_else(|| {
        let s = step.abs();
        if (s - 1.0).abs() < 1e-9 {
            0
        } else if (s - 0.1).abs() < 1e-9 {
            1
        } else if (s - 0.01).abs() < 1e-9 {
            2
        } else if (s - 0.001).abs() < 1e-9 {
            3
        } else {
            3
        }
    });

    let val = match unit {
        DisplayUnit::Mm => round_step(mm, step),
        DisplayUnit::Inch => round_step(mm_to_inch(mm), step / 25.4),
    };

    format!("{:.*}", prec as usize, val)
}

pub fn fmt_angle_deg(style: &StylePreset, deg: f64, precision_override: Option<u8>) -> String {
    let step = style.rounding.angle_deg_step;
    let val = round_step(deg, step);
    let prec = precision_override.unwrap_or_else(|| {
        let s = step.abs();
        if (s - 1.0).abs() < 1e-9 {
            0
        } else if (s - 0.5).abs() < 1e-9 || (s - 0.1).abs() < 1e-9 {
            1
        } else {
            2
        }
    });
    format!("{:.*}°", prec as usize, val)
}

pub fn fmt_diameter(style: &StylePreset, mm: f64, precision_override: Option<u8>) -> String {
    format!("⌀{}", fmt_length(style, mm, precision_override))
}

pub fn fmt_radius(style: &StylePreset, mm: f64, precision_override: Option<u8>) -> String {
    format!("R{}", fmt_length(style, mm, precision_override))
}

pub fn fmt_chamfer_c(style: &StylePreset, mm: f64, precision_override: Option<u8>) -> String {
    format!("C{}", fmt_length(style, mm, precision_override))
}
