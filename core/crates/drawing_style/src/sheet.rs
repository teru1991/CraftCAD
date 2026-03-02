use crate::render_ir::*;
use crate::ssot::{DrawingSsotBundle, RectMm};

#[derive(Debug, Clone)]
pub struct ProjectMeta {
    pub project_title: String,
    pub drawing_title: String,
    pub scale: String,
    pub unit: String,
    pub date: String,
    pub author: String,
    pub revision: String,
    pub schema_version: String,
    pub app_version: String,
}

#[derive(Debug, Clone)]
pub struct KeepOutZones {
    pub title_block: Rect,
    pub outside_border: Rect,
    pub model_view: Rect,
    pub page: Rect,
    pub border: Rect,
}

fn rect_from_mm(r: &RectMm) -> Rect {
    Rect {
        x: Mm(r.x_mm),
        y: Mm(r.y_mm),
        w: Mm(r.w_mm),
        h: Mm(r.h_mm),
    }
}

pub fn compute_keepouts(bundle: &DrawingSsotBundle) -> KeepOutZones {
    let page = &bundle.sheet.page;
    let m = &page.margin_mm;
    let border = Rect {
        x: Mm(m.left),
        y: Mm(m.top),
        w: Mm(page.width_mm - m.left - m.right),
        h: Mm(page.height_mm - m.top - m.bottom),
    };
    KeepOutZones {
        title_block: rect_from_mm(&page.title_block.bbox_mm),
        outside_border: Rect {
            x: Mm(0.0),
            y: Mm(0.0),
            w: Mm(page.width_mm),
            h: Mm(page.height_mm),
        },
        model_view: rect_from_mm(&bundle.sheet.viewports.model_view_region),
        page: Rect {
            x: Mm(0.0),
            y: Mm(0.0),
            w: Mm(page.width_mm),
            h: Mm(page.height_mm),
        },
        border,
    }
}

pub fn build_sheet_ir(bundle: &DrawingSsotBundle, meta: &ProjectMeta) -> RenderIr {
    let page = &bundle.sheet.page;
    let mut ir = RenderIr::new(page.width_mm, page.height_mm);

    let stroke = StrokeStyle {
        width_mm: bundle
            .style
            .line_weights
            .normal_mm
            .max(bundle.style.line_weights.min_line_weight_mm),
        color_hex: bundle.style.colors.default_stroke_hex.clone(),
        dash_pattern_mm: vec![],
    };

    let keepouts = compute_keepouts(bundle);

    ir.push(StyledPrimitive {
        z: 0,
        stable_id: "SHEET_BORDER".to_string(),
        stroke: Some(stroke.clone()),
        fill: None,
        text: None,
        prim: Primitive::Rect {
            rect: keepouts.border,
        },
    });

    ir.push(StyledPrimitive {
        z: 0,
        stable_id: "TITLE_BLOCK_BORDER".to_string(),
        stroke: Some(stroke.clone()),
        fill: None,
        text: None,
        prim: Primitive::Rect {
            rect: keepouts.title_block,
        },
    });

    let fs = page.title_block.field_font_size_mm;
    let text_style = TextStyle {
        font_family: bundle.style.fonts.primary_family.clone(),
        fallback_families: bundle.style.fonts.fallback_families.clone(),
        size_mm: fs,
        color_hex: bundle.style.colors.default_stroke_hex.clone(),
        kerning: bundle.print.text_hinting.kerning,
    };

    let line_h = fs * 1.35;
    let mut cursor_y = keepouts.title_block.y.0 + fs * 1.2;
    let left_x = keepouts.title_block.x.0 + fs * 0.8;

    for f in &page.title_block.fields {
        let val = match f.key.as_str() {
            "project_title" => &meta.project_title,
            "drawing_title" => &meta.drawing_title,
            "scale" => &meta.scale,
            "unit" => &meta.unit,
            "date" => &meta.date,
            "author" => &meta.author,
            "revision" => &meta.revision,
            "schema_version" => &meta.schema_version,
            "app_version" => &meta.app_version,
            _ => "",
        };

        let label = f
            .label
            .ja
            .as_deref()
            .or(f.label.en.as_deref())
            .unwrap_or(&f.key);
        let mut s = format!("{label}: {val}");

        if let Some(maxc) = f.max_chars {
            if s.chars().count() as u32 > maxc {
                s = s.chars().take(maxc as usize).collect::<String>();
            }
        }

        if cursor_y <= keepouts.title_block.y.0 + keepouts.title_block.h.0 - fs * 0.6 {
            let bbox = estimate_text_bbox_mm(
                Pt2 {
                    x: Mm(left_x),
                    y: Mm(cursor_y),
                },
                &s,
                fs,
                "start",
                0.0,
            );
            ir.push(StyledPrimitive {
                z: 1,
                stable_id: format!("TITLE_{}", f.key),
                stroke: None,
                fill: None,
                text: Some(text_style.clone()),
                prim: Primitive::Text {
                    pos: Pt2 {
                        x: Mm(left_x),
                        y: Mm(cursor_y),
                    },
                    text: s,
                    rotation_deg: 0.0,
                    anchor: "start".to_string(),
                    bbox_hint_mm: Some(bbox),
                },
            });
        }
        cursor_y += line_h;
    }

    ir.push(StyledPrimitive {
        z: 0,
        stable_id: "MODEL_VIEW_CLIP".to_string(),
        stroke: None,
        fill: None,
        text: None,
        prim: Primitive::ClipRect {
            rect: keepouts.model_view,
        },
    });

    ir.sort_stable();
    ir
}
