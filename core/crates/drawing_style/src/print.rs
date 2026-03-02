use crate::render_ir::{Mm, Pt2, Rect, RenderIr};
use crate::ssot::{DrawingSsotBundle, PrintPreset};

#[derive(Debug, Clone)]
pub struct PrintPlacement {
    pub scale: f64,
    pub translate_mm: (f64, f64),
    pub warned_min_text_height: bool,
}

pub fn compute_print_placement(bundle: &DrawingSsotBundle, model_bbox: Rect) -> PrintPlacement {
    compute_print_placement_with_preset(&bundle.print, bundle, model_bbox)
}

pub fn compute_print_placement_with_preset(
    preset: &PrintPreset,
    bundle: &DrawingSsotBundle,
    model_bbox: Rect,
) -> PrintPlacement {
    let mv = &bundle.sheet.viewports.model_view_region;
    let sx = if model_bbox.w.0.abs() < 1e-9 {
        1.0
    } else {
        mv.w_mm / model_bbox.w.0
    };
    let sy = if model_bbox.h.0.abs() < 1e-9 {
        1.0
    } else {
        mv.h_mm / model_bbox.h.0
    };
    let mut scale = match preset.scale_policy.mode.as_str() {
        "fixed" => preset.scale_policy.fixed_scale.unwrap_or(1.0),
        _ => sx.min(sy),
    };

    let mut warned = false;
    if bundle.style.fonts.size_mm < preset.min_readable_text_height_mm {
        warned = true;
        scale = scale.max(1.0);
    }

    let w = model_bbox.w.0 * scale;
    let h = model_bbox.h.0 * scale;
    let tx = mv.x_mm + (mv.w_mm - w) * 0.5 - model_bbox.x.0 * scale;
    let ty = mv.y_mm + (mv.h_mm - h) * 0.5 - model_bbox.y.0 * scale;

    PrintPlacement {
        scale,
        translate_mm: (tx, ty),
        warned_min_text_height: warned,
    }
}

pub fn apply_bw_mode(ir: &mut RenderIr) {
    for it in &mut ir.items {
        if let Some(st) = &mut it.stroke {
            st.color_hex = "#000000".to_string();
        }
        if let Some(tx) = &mut it.text {
            tx.color_hex = "#000000".to_string();
        }
        if let Some(f) = &mut it.fill {
            f.color_hex = "#ffffff".to_string();
        }
    }
}

pub fn apply_line_weight_scale(ir: &mut RenderIr, scale: f64, min_line_weight_mm: f64) {
    let minw = min_line_weight_mm.max(0.03);
    for it in &mut ir.items {
        if let Some(st) = &mut it.stroke {
            let w = st.width_mm * scale;
            st.width_mm = w.max(minw);
        }
    }
}

pub fn transform_pt(scale: f64, translate: (f64, f64), p: Pt2) -> Pt2 {
    Pt2 {
        x: Mm(p.x.0 * scale + translate.0),
        y: Mm(p.y.0 * scale + translate.1),
    }
}
