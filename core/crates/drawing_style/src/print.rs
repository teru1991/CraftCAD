use crate::render_ir::RenderIr;

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
