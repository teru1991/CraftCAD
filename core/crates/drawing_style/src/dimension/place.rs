use super::types::*;
use crate::render_ir::*;
use crate::ssot::DrawingSsotBundle;
use crate::units::{fmt_angle_deg, fmt_diameter, fmt_length, fmt_radius};

fn mmpt(x: f64, y: f64) -> Pt2 {
    Pt2 { x: Mm(x), y: Mm(y) }
}

fn text_bbox_at(pos: (f64, f64), text: &str, size_mm: f64, pad_mm: f64) -> Rect {
    let w = (text.chars().count() as f64) * size_mm * 0.6;
    let h = size_mm * 1.2;
    Rect {
        x: Mm(pos.0 - pad_mm),
        y: Mm(pos.1 - h - pad_mm),
        w: Mm(w + 2.0 * pad_mm),
        h: Mm(h + 2.0 * pad_mm),
    }
}

pub fn place_dimension(
    bundle: &DrawingSsotBundle,
    measured: &MeasuredDimension,
    hint: &PlacementHint,
    ov: &DimensionOverrides,
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
    match measured.kind {
        DimensionKind::LinearSerial | DimensionKind::LinearBaseline => {
            let p0 = measured.anchor_points_mm[0];
            let p1 = measured.anchor_points_mm[1];
            let dx = p1.0 - p0.0;
            let dy = p1.1 - p0.1;
            let len = (dx * dx + dy * dy).sqrt().max(1e-9);
            let nx = -dy / len;
            let ny = dx / len;
            let base = dim.dim_line_offset_mm + (hint.offset_level as f64) * dim.dim_line_step_mm;
            let sign = match hint.side {
                Side::Right | Side::Bottom => -1.0,
                _ => 1.0,
            };
            let q0 = (p0.0 + nx * base * sign, p0.1 + ny * base * sign);
            let q1 = (p1.0 + nx * base * sign, p1.1 + ny * base * sign);
            items.push(StyledPrimitive {
                z: 30,
                stable_id: "DIM_EXT_0".to_string(),
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
                stable_id: "DIM_EXT_1".to_string(),
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
                stable_id: "DIM_LINE".to_string(),
                stroke: Some(stroke.clone()),
                fill: None,
                text: None,
                prim: Primitive::Line {
                    a: mmpt(q0.0, q0.1),
                    b: mmpt(q1.0, q1.1),
                },
            });

            let a = dim.arrow.size_mm;
            let ux = dx / len;
            let uy = dy / len;
            let draw_arrow = |id: &str, tip: (f64, f64), dir: (f64, f64)| {
                let ang = 30.0_f64.to_radians();
                let (c, s) = (ang.cos(), ang.sin());
                let r1 = (dir.0 * c - dir.1 * s, dir.0 * s + dir.1 * c);
                let r2 = (dir.0 * c + dir.1 * s, -dir.0 * s + dir.1 * c);
                vec![
                    StyledPrimitive {
                        z: 32,
                        stable_id: format!("{}_A", id),
                        stroke: Some(stroke.clone()),
                        fill: None,
                        text: None,
                        prim: Primitive::Line {
                            a: mmpt(tip.0, tip.1),
                            b: mmpt(tip.0 - r1.0 * a, tip.1 - r1.1 * a),
                        },
                    },
                    StyledPrimitive {
                        z: 32,
                        stable_id: format!("{}_B", id),
                        stroke: Some(stroke.clone()),
                        fill: None,
                        text: None,
                        prim: Primitive::Line {
                            a: mmpt(tip.0, tip.1),
                            b: mmpt(tip.0 - r2.0 * a, tip.1 - r2.1 * a),
                        },
                    },
                ]
            };
            items.extend(draw_arrow("ARROW_0", q0, (ux, uy)));
            items.extend(draw_arrow("ARROW_1", q1, (-ux, -uy)));
            let mid = ((q0.0 + q1.0) / 2.0, (q0.1 + q1.1) / 2.0);
            let tpos = hint.manual_text_pos_mm.unwrap_or((
                mid.0 + nx * dim.text_gap_mm * sign,
                mid.1 + ny * dim.text_gap_mm * sign,
            ));
            let bbox = text_bbox_at(tpos, &text, st.fonts.size_mm, dim.text_box_padding_mm);
            items.push(StyledPrimitive {
                z: 40,
                stable_id: "DIM_TEXT".to_string(),
                stroke: None,
                fill: None,
                text: Some(text_style),
                prim: Primitive::Text {
                    pos: mmpt(tpos.0, tpos.1),
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
        DimensionKind::Angular => {
            let p0 = measured.anchor_points_mm[0];
            let v = measured.anchor_points_mm[1];
            let p2 = measured.anchor_points_mm[2];
            let base = dim.dim_line_offset_mm + (hint.offset_level as f64) * dim.dim_line_step_mm;
            let tpos = hint.manual_text_pos_mm.unwrap_or((v.0 + base, v.1 - base));
            let bbox = text_bbox_at(tpos, &text, st.fonts.size_mm, dim.text_box_padding_mm);
            items.push(StyledPrimitive {
                z: 30,
                stable_id: "ANG_LEG_0".to_string(),
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
                stable_id: "ANG_LEG_1".to_string(),
                stroke: Some(stroke),
                fill: None,
                text: None,
                prim: Primitive::Line {
                    a: mmpt(v.0, v.1),
                    b: mmpt(p2.0, p2.1),
                },
            });
            items.push(StyledPrimitive {
                z: 40,
                stable_id: "ANG_TEXT".to_string(),
                stroke: None,
                fill: None,
                text: Some(text_style),
                prim: Primitive::Text {
                    pos: mmpt(tpos.0, tpos.1),
                    text,
                    rotation_deg: 0.0,
                    anchor: "start".to_string(),
                    bbox_hint_mm: Some(bbox),
                },
            });
            PlacedDimensionIr {
                items,
                text_bbox_mm: Some(bbox),
            }
        }
        DimensionKind::Radius | DimensionKind::Diameter => {
            let c = measured.anchor_points_mm[0];
            let p = measured.anchor_points_mm[1];
            items.push(StyledPrimitive {
                z: 30,
                stable_id: "RAD_LEADER".to_string(),
                stroke: Some(stroke),
                fill: None,
                text: None,
                prim: Primitive::Line {
                    a: mmpt(c.0, c.1),
                    b: mmpt(p.0, p.1),
                },
            });
            let base = dim.dim_line_offset_mm + (hint.offset_level as f64) * dim.dim_line_step_mm;
            let tpos = hint.manual_text_pos_mm.unwrap_or((p.0 + base, p.1 - base));
            let bbox = text_bbox_at(tpos, &text, st.fonts.size_mm, dim.text_box_padding_mm);
            items.push(StyledPrimitive {
                z: 40,
                stable_id: "RAD_TEXT".to_string(),
                stroke: None,
                fill: None,
                text: Some(text_style),
                prim: Primitive::Text {
                    pos: mmpt(tpos.0, tpos.1),
                    text,
                    rotation_deg: 0.0,
                    anchor: "start".to_string(),
                    bbox_hint_mm: Some(bbox),
                },
            });
            PlacedDimensionIr {
                items,
                text_bbox_mm: Some(bbox),
            }
        }
    }
}
