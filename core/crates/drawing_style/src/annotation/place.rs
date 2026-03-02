use super::leader::leader_ir;
use super::types::*;
use crate::annotation::{chamfer::chamfer_text, hole::hole_text};
use crate::render_ir::*;
use crate::sheet::KeepOutZones;
use crate::ssot::DrawingSsotBundle;

fn mmpt(x: f64, y: f64) -> Pt2 {
    Pt2 { x: Mm(x), y: Mm(y) }
}

pub fn place_annotation(
    bundle: &DrawingSsotBundle,
    stable_id: &str,
    _kind: AnnotationKind,
    anchor: (f64, f64),
    payload: &AnnotationPayload,
    keepouts: &KeepOutZones,
    other_text_bboxes: &[Rect],
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
    let bbox = estimate_text_bbox_mm(
        mmpt(tpos.0, tpos.1),
        &text,
        st.fonts.size_mm,
        "start",
        dim.text_box_padding_mm,
    );
    let mut items = vec![];

    if leader_hint.is_some() {
        items.extend(leader_ir(
            bundle,
            stable_id,
            anchor,
            tpos,
            bbox,
            keepouts,
            other_text_bboxes,
        ));
    }

    items.push(StyledPrimitive {
        z: 44,
        stable_id: format!("{stable_id}_BG"),
        stroke: None,
        fill: Some(FillStyle {
            color_hex: "#ffffff".to_string(),
        }),
        text: None,
        prim: Primitive::Rect { rect: bbox },
    });
    items.push(StyledPrimitive {
        z: 45,
        stable_id: format!("{stable_id}_TEXT"),
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
