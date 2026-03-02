use crate::render_ir::*;
use crate::ssot::DrawingSsotBundle;

fn mmpt(x: f64, y: f64) -> Pt2 {
    Pt2 { x: Mm(x), y: Mm(y) }
}

pub fn leader_ir(
    bundle: &DrawingSsotBundle,
    stable_prefix: &str,
    anchor: (f64, f64),
    bend: (f64, f64),
    text_pos: (f64, f64),
) -> Vec<StyledPrimitive> {
    let st = &bundle.style;
    let stroke = StrokeStyle {
        width_mm: st
            .line_weights
            .thin_mm
            .max(st.line_weights.min_line_weight_mm),
        color_hex: st.colors.default_stroke_hex.clone(),
        dash_pattern_mm: vec![],
    };
    vec![
        StyledPrimitive {
            z: 35,
            stable_id: format!("{}_L0", stable_prefix),
            stroke: Some(stroke.clone()),
            fill: None,
            text: None,
            prim: Primitive::Line {
                a: mmpt(anchor.0, anchor.1),
                b: mmpt(bend.0, bend.1),
            },
        },
        StyledPrimitive {
            z: 35,
            stable_id: format!("{}_L1", stable_prefix),
            stroke: Some(stroke),
            fill: None,
            text: None,
            prim: Primitive::Line {
                a: mmpt(bend.0, bend.1),
                b: mmpt(text_pos.0, text_pos.1),
            },
        },
    ]
}
