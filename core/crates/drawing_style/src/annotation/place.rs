use super::leader::leader_ir;
use super::types::*;
use crate::annotation::{chamfer::chamfer_text, hole::hole_text};
use crate::render_ir::*;
use crate::ssot::DrawingSsotBundle;

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

pub fn place_annotation(
    bundle: &DrawingSsotBundle,
    stable_id: &str,
    _kind: AnnotationKind,
    anchor: (f64, f64),
    payload: &AnnotationPayload,
) -> (Vec<StyledPrimitive>, Option<Rect>) {
    let st = &bundle.style;
    let dim = &st.dimension;
    let text_style = TextStyle {
        font_family: st.fonts.primary_family.clone(),
        fallback_families: st.fonts.fallback_families.clone(),
        size_mm: st.fonts.size_mm,
        color_hex: st.colors.default_stroke_hex.clone(),
        kerning: bundle.print.text_hinting.kerning,
    };

    let (text, leader_hint) = match payload {
        AnnotationPayload::Text { text } => (text.clone(), None),
        AnnotationPayload::LeaderText { text, leader } => (text.clone(), Some(leader.clone())),
        AnnotationPayload::Hole { info, leader } => (
            hole_text(st, info.diameter_mm, info.count),
            Some(leader.clone()),
        ),
        AnnotationPayload::Chamfer { info, leader } => (
            chamfer_text(
                st,
                match info.ty {
                    ChamferType::C => "C",
                    ChamferType::R => "R",
                },
                info.value_mm,
            ),
            Some(leader.clone()),
        ),
    };

    let tpos = leader_hint.as_ref().and_then(|h| h.text_pos_mm).unwrap_or((
        anchor.0 + dim.dim_line_offset_mm,
        anchor.1 - dim.dim_line_offset_mm,
    ));
    let bbox = text_bbox_at(tpos, &text, st.fonts.size_mm, dim.text_box_padding_mm);
    let mut items = vec![];

    if let Some(lh) = leader_hint {
        let bend = lh.bend_mm.unwrap_or_else(|| {
            let ang = lh.default_angle_deg.to_radians();
            (
                anchor.0 + ang.cos() * dim.dim_line_offset_mm,
                anchor.1 - ang.sin() * dim.dim_line_offset_mm,
            )
        });
        items.extend(leader_ir(bundle, stable_id, anchor, bend, tpos));
    }

    items.push(StyledPrimitive {
        z: 45,
        stable_id: format!("{}_TEXT", stable_id),
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

    (items, Some(bbox))
}
