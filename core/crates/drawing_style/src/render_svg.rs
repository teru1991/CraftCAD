use crate::render_ir::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SvgError {
    #[error("invalid svg precision: {0}")]
    InvalidPrecision(u32),
}

fn fmt_f(v: f64, prec: u32) -> String {
    format!("{:.*}", prec.min(8) as usize, v)
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn normalize_svg_for_golden(svg: &str) -> String {
    svg.lines().map(str::trim).collect::<Vec<_>>().join("\n") + "\n"
}

pub fn render_svg(ir: &RenderIr, precision: u32) -> Result<String, SvgError> {
    if precision > 8 {
        return Err(SvgError::InvalidPrecision(precision));
    }

    let mut items = ir.items.clone();
    items.sort_by(|a, b| {
        let z = a.z.cmp(&b.z);
        if z != std::cmp::Ordering::Equal {
            return z;
        }
        a.stable_id.cmp(&b.stable_id)
    });

    let mut out = String::new();
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push('\n');
    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}mm" height="{}mm" viewBox="0 0 {} {}">"#,
        fmt_f(ir.page_w_mm, precision),
        fmt_f(ir.page_h_mm, precision),
        fmt_f(ir.page_w_mm, precision),
        fmt_f(ir.page_h_mm, precision)
    ));
    out.push('\n');

    for it in items {
        if let Primitive::ClipRect { rect } = it.prim {
            out.push_str(&format!(
                r#"<!-- CLIP {} x={} y={} w={} h={} -->"#,
                it.stable_id,
                fmt_f(rect.x.0, precision),
                fmt_f(rect.y.0, precision),
                fmt_f(rect.w.0, precision),
                fmt_f(rect.h.0, precision)
            ));
            out.push('\n');
            continue;
        }

        let mut attrs: Vec<String> = vec![];
        if let Some(f) = &it.fill {
            attrs.push(format!(r#"fill="{}""#, f.color_hex));
        } else {
            attrs.push(r#"fill="none""#.to_string());
        }
        if let Some(st) = &it.stroke {
            attrs.push(format!(r#"stroke="{}""#, st.color_hex));
            attrs.push(format!(
                r#"stroke-width="{}""#,
                fmt_f(st.width_mm, precision)
            ));
            if !st.dash_pattern_mm.is_empty() {
                let pat = st
                    .dash_pattern_mm
                    .iter()
                    .map(|x| fmt_f(*x, precision))
                    .collect::<Vec<_>>()
                    .join(",");
                attrs.push(format!(r#"stroke-dasharray="{}""#, pat));
            }
        }

        match it.prim {
            Primitive::Line { a, b } => out.push_str(&format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" {} />"#,
                fmt_f(a.x.0, precision),
                fmt_f(a.y.0, precision),
                fmt_f(b.x.0, precision),
                fmt_f(b.y.0, precision),
                attrs.join(" ")
            )),
            Primitive::Rect { rect } => out.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" {} />"#,
                fmt_f(rect.x.0, precision),
                fmt_f(rect.y.0, precision),
                fmt_f(rect.w.0, precision),
                fmt_f(rect.h.0, precision),
                attrs.join(" ")
            )),
            Primitive::Circle { c, r } => out.push_str(&format!(
                r#"<circle cx="{}" cy="{}" r="{}" {} />"#,
                fmt_f(c.x.0, precision),
                fmt_f(c.y.0, precision),
                fmt_f(r.0, precision),
                attrs.join(" ")
            )),
            Primitive::Arc {
                c,
                r,
                start_deg,
                sweep_deg,
            } => {
                let start_rad = start_deg.to_radians();
                let end_rad = (start_deg + sweep_deg).to_radians();
                let sx = c.x.0 + r.0 * start_rad.cos();
                let sy = c.y.0 + r.0 * start_rad.sin();
                let ex = c.x.0 + r.0 * end_rad.cos();
                let ey = c.y.0 + r.0 * end_rad.sin();
                let large_arc = if sweep_deg.abs() > 180.0 { 1 } else { 0 };
                let sweep_flag = if sweep_deg >= 0.0 { 1 } else { 0 };
                out.push_str(&format!(
                    r#"<path d="M {} {} A {} {} 0 {} {} {} {}" {} />"#,
                    fmt_f(sx, precision),
                    fmt_f(sy, precision),
                    fmt_f(r.0, precision),
                    fmt_f(r.0, precision),
                    large_arc,
                    sweep_flag,
                    fmt_f(ex, precision),
                    fmt_f(ey, precision),
                    attrs.join(" ")
                ))
            }
            Primitive::Polyline { pts, closed } => {
                let mut p = pts
                    .iter()
                    .map(|pt| format!("{},{}", fmt_f(pt.x.0, precision), fmt_f(pt.y.0, precision)))
                    .collect::<Vec<_>>()
                    .join(" ");
                if closed {
                    if let Some(first) = pts.first() {
                        p.push_str(&format!(
                            " {},{}",
                            fmt_f(first.x.0, precision),
                            fmt_f(first.y.0, precision)
                        ));
                    }
                }
                out.push_str(&format!(
                    r#"<polyline points="{}" {} />"#,
                    p,
                    attrs.join(" ")
                ))
            }
            Primitive::Text {
                pos,
                text,
                rotation_deg,
                anchor,
                ..
            } => {
                let tx = it
                    .text
                    .as_ref()
                    .expect("text style required for Text primitive");
                let mut fams = vec![tx.font_family.clone()];
                fams.extend(tx.fallback_families.clone());
                let family = fams.join(", ");
                let escaped = escape_text(&text);

                let mut tattrs = vec![
                    format!(r#"x="{}""#, fmt_f(pos.x.0, precision)),
                    format!(r#"y="{}""#, fmt_f(pos.y.0, precision)),
                    format!(r#"font-family="{}""#, family),
                    format!(r#"font-size="{}mm""#, fmt_f(tx.size_mm, precision)),
                    format!(r#"text-anchor="{}""#, anchor),
                    format!(r#"fill="{}""#, tx.color_hex),
                ];
                if rotation_deg.abs() > 1e-9 {
                    tattrs.push(format!(
                        r#"transform="rotate({} {} {})""#,
                        fmt_f(rotation_deg, precision),
                        fmt_f(pos.x.0, precision),
                        fmt_f(pos.y.0, precision)
                    ));
                }
                out.push_str(&format!(r#"<text {}>{}</text>"#, tattrs.join(" "), escaped));
            }
            Primitive::ClipRect { .. } => unreachable!(),
        }
        out.push('\n');
    }

    out.push_str("</svg>\n");
    Ok(out)
}
