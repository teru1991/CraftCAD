use super::leader_layout::{solve_leader_layout, LeaderLayoutInput};
use crate::render_ir::*;
use crate::sheet::KeepOutZones;
use crate::ssot::DrawingSsotBundle;

fn mmpt(x: f64, y: f64) -> Pt2 {
    Pt2 { x: Mm(x), y: Mm(y) }
}

pub fn leader_ir(
    bundle: &DrawingSsotBundle,
    stable_prefix: &str,
    anchor: (f64, f64),
    text_pos: (f64, f64),
    text_bbox: Rect,
    keepouts: &KeepOutZones,
    other_text_bboxes: &[Rect],
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
    let out = solve_leader_layout(&LeaderLayoutInput {
        ann_id: stable_prefix.to_string(),
        anchor: mmpt(anchor.0, anchor.1),
        text_pos: mmpt(text_pos.0, text_pos.1),
        text_bbox,
        other_text_bboxes: other_text_bboxes.to_vec(),
        keepouts: keepouts.clone(),
        max_variants: 20,
    });
    let _congested = out.congested;
    vec![
        StyledPrimitive {
            z: 35,
            stable_id: format!("{}_L0", stable_prefix),
            stroke: Some(stroke.clone()),
            fill: None,
            text: None,
            prim: Primitive::Line {
                a: mmpt(anchor.0, anchor.1),
                b: out.elbow,
            },
        },
        StyledPrimitive {
            z: 35,
            stable_id: format!("{}_L1", stable_prefix),
            stroke: Some(stroke),
            fill: None,
            text: None,
            prim: Primitive::Line {
                a: out.elbow,
                b: mmpt(text_pos.0, text_pos.1),
            },
        },
    ]
}
