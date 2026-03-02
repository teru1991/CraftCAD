use super::layout::{solve_layout, LayoutInput};
use super::types::*;
use crate::render_ir::*;
use crate::sheet::KeepOutZones;
use crate::ssot::DrawingSsotBundle;
use crate::units::{fmt_angle_deg, fmt_diameter, fmt_length, fmt_radius};

fn mmpt(x: f64, y: f64) -> Pt2 {
    Pt2 { x: Mm(x), y: Mm(y) }
}

pub fn place_dimension(
    bundle: &DrawingSsotBundle,
    dim_id: &str,
    measured: &MeasuredDimension,
    hint: &PlacementHint,
    ov: &DimensionOverrides,
    keepouts: &KeepOutZones,
    other_text_bboxes: &[Rect],
) -> PlacedDimensionIr {
    let st = &bundle.style;
    let dim = &st.dimension;

    let text_style = TextStyle {
        font_family: st.fonts.primary_family.clone(),
        fallback_families: st.fonts.fallback_families.clone(),
        size_mm: st.fonts.size_mm,
        color_hex: st.colors.default_stroke_hex.clone(),
        kerning: bundle.print.text_hinting.kerning,
    };
    let stroke = StrokeStyle {
        width_mm: st
            .line_weights
            .thin_mm
            .max(st.line_weights.min_line_weight_mm),
        color_hex: st.colors.default_stroke_hex.clone(),
        dash_pattern_mm: vec![],
    };

    let text = ov
        .text_override
        .clone()
        .unwrap_or_else(|| match measured.kind {
            DimensionKind::Angular => {
                fmt_angle_deg(st, measured.value_deg.unwrap_or(0.0), ov.precision_override)
            }
            DimensionKind::Diameter => fmt_diameter(st, measured.value_mm, ov.precision_override),
            DimensionKind::Radius => fmt_radius(st, measured.value_mm, ov.precision_override),
            _ => fmt_length(st, measured.value_mm, ov.precision_override),
        });

    let mut items = vec![];
    let p0 = measured.anchor_points_mm[0];
    let p1 = measured.anchor_points_mm.get(1).copied().unwrap_or(p0);
    let dx = p1.0 - p0.0;
    let dy = p1.1 - p0.1;
    let len = (dx * dx + dy * dy).sqrt().max(1e-9);
    let nx = -dy / len;
    let ny = dx / len;
    let mut sign = match hint.side {
        Side::Right | Side::Bottom => -1.0,
        _ => 1.0,
    };
    let mid = ((p0.0 + p1.0) / 2.0, (p0.1 + p1.1) / 2.0);
    let base_text_pos = hint.manual_text_pos_mm.unwrap_or((
        mid.0 + nx * dim.text_gap_mm * sign,
        mid.1 + ny * dim.text_gap_mm * sign,
    ));
    let base_pt = mmpt(base_text_pos.0, base_text_pos.1);
    let base_bbox = estimate_text_bbox_mm(
        base_pt,
        &text,
        st.fonts.size_mm,
        "middle",
        dim.text_box_padding_mm,
    );

    let layout = solve_layout(
        &LayoutInput {
            dim_id: dim_id.to_string(),
            base_text_pos: base_pt,
            manual_text_pos: hint.manual_text_pos_mm.map(|(x, y)| mmpt(x, y)),
            base_offset_level: hint.offset_level,
            allow_flip_side: true,
            max_extra_levels: 3,
            text_bbox: base_bbox,
            other_text_bboxes: other_text_bboxes.to_vec(),
            keepouts: keepouts.clone(),
        },
        |p| {
            estimate_text_bbox_mm(
                p,
                &text,
                st.fonts.size_mm,
                "middle",
                dim.text_box_padding_mm,
            )
        },
    );

    if layout.used_flip {
        sign *= -1.0;
    }
    let base = dim.dim_line_offset_mm + (layout.chosen_offset_level as f64) * dim.dim_line_step_mm;

    match measured.kind {
        DimensionKind::LinearSerial | DimensionKind::LinearBaseline => {
            let q0 = (p0.0 + nx * base * sign, p0.1 + ny * base * sign);
            let q1 = (p1.0 + nx * base * sign, p1.1 + ny * base * sign);
            items.push(StyledPrimitive {
                z: 30,
                stable_id: "DIM_EXT_0".into(),
                stroke: Some(stroke.clone()),
                fill: None,
                text: None,
                prim: Primitive::Line {
                    a: mmpt(p0.0, p0.1),
                    b: mmpt(q0.0, q0.1),
                },
            });
            items.push(StyledPrimitive {
                z: 30,
                stable_id: "DIM_EXT_1".into(),
                stroke: Some(stroke.clone()),
                fill: None,
                text: None,
                prim: Primitive::Line {
                    a: mmpt(p1.0, p1.1),
                    b: mmpt(q1.0, q1.1),
                },
            });
            items.push(StyledPrimitive {
                z: 31,
                stable_id: "DIM_LINE".into(),
                stroke: Some(stroke.clone()),
                fill: None,
                text: None,
                prim: Primitive::Line {
                    a: mmpt(q0.0, q0.1),
                    b: mmpt(q1.0, q1.1),
                },
            });
        }
        DimensionKind::Angular => {
            let v = measured.anchor_points_mm[1];
            let p2 = measured.anchor_points_mm[2];
            items.push(StyledPrimitive {
                z: 30,
                stable_id: "ANG_LEG_0".into(),
                stroke: Some(stroke.clone()),
                fill: None,
                text: None,
                prim: Primitive::Line {
                    a: mmpt(v.0, v.1),
                    b: mmpt(p0.0, p0.1),
                },
            });
            items.push(StyledPrimitive {
                z: 30,
                stable_id: "ANG_LEG_1".into(),
                stroke: Some(stroke.clone()),
                fill: None,
                text: None,
                prim: Primitive::Line {
                    a: mmpt(v.0, v.1),
                    b: mmpt(p2.0, p2.1),
                },
            });
        }
        DimensionKind::Radius | DimensionKind::Diameter => {
            items.push(StyledPrimitive {
                z: 30,
                stable_id: "RAD_LEADER".into(),
                stroke: Some(stroke.clone()),
                fill: None,
                text: None,
                prim: Primitive::Line {
                    a: mmpt(p0.0, p0.1),
                    b: mmpt(p1.0, p1.1),
                },
            });
        }
    }

    let tpos = layout.chosen_text_pos;
    let bbox = estimate_text_bbox_mm(
        tpos,
        &text,
        st.fonts.size_mm,
        "middle",
        dim.text_box_padding_mm,
    );
    items.push(StyledPrimitive {
        z: 39,
        stable_id: "DIM_TEXT_BG".to_string(),
        stroke: None,
        fill: Some(FillStyle {
            color_hex: "#ffffff".to_string(),
        }),
        text: None,
        prim: Primitive::Rect { rect: bbox },
    });
    items.push(StyledPrimitive {
        z: 40,
        stable_id: "DIM_TEXT".to_string(),
        stroke: None,
        fill: None,
        text: Some(text_style),
        prim: Primitive::Text {
            pos: tpos,
            text,
            rotation_deg: 0.0,
            anchor: "middle".to_string(),
            bbox_hint_mm: Some(bbox),
        },
    });

    PlacedDimensionIr {
        items,
        text_bbox_mm: Some(bbox),
    }
}
