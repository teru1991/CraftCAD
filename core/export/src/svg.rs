use craftcad_serialize::{Document, Geom2D, Reason, ReasonCode, Result, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvgExportOptions {
    pub precision: usize,
    pub include_parts: bool,
    pub include_entities: bool,
}
impl Default for SvgExportOptions {
    fn default() -> Self {
        Self {
            precision: 3,
            include_parts: true,
            include_entities: true,
        }
    }
}

fn fmt(v: f64, p: usize) -> String {
    format!("{:.1$}", v, p)
}
fn poly_path(pts: &[Vec2], closed: bool, prec: usize) -> String {
    let mut out = String::new();
    if let Some(first) = pts.first() {
        out.push_str(&format!("M {} {}", fmt(first.x, prec), fmt(first.y, prec)));
        for p in pts.iter().skip(1) {
            out.push_str(&format!(" L {} {}", fmt(p.x, prec), fmt(p.y, prec)));
        }
        if closed {
            out.push_str(" Z");
        }
    }
    out
}

pub fn export_svg(doc: &Document, options: &SvgExportOptions) -> Result<String> {
    let mut items: Vec<String> = vec![];
    if options.include_entities {
        let mut entities = doc.entities.clone();
        entities.sort_by_key(|e| e.id);
        for e in entities {
            match e.geom {
                Geom2D::Line { a, b } => {
                    items.push(format!(
                        "<path data-id=\"{}\" d=\"M {} {} L {} {}\" class=\"entity line\" />",
                        e.id,
                        fmt(a.x, options.precision),
                        fmt(a.y, options.precision),
                        fmt(b.x, options.precision),
                        fmt(b.y, options.precision)
                    ));
                }
                Geom2D::Polyline { pts, closed } => {
                    items.push(format!(
                        "<path data-id=\"{}\" d=\"{}\" class=\"entity polyline\" />",
                        e.id,
                        poly_path(&pts, closed, options.precision)
                    ));
                }
                _ => return Err(Reason::from_code(ReasonCode::ExportUnsupportedEntity)),
            }
        }
    }
    if options.include_parts {
        let mut parts = doc.parts.clone();
        parts.sort_by_key(|p| p.id);
        for p in parts {
            items.push(format!(
                "<path data-part-id=\"{}\" d=\"{}\" class=\"part outer\" />",
                p.id,
                poly_path(&p.outline.outer, true, options.precision)
            ));
            for h in p.outline.holes {
                items.push(format!(
                    "<path data-part-id=\"{}\" d=\"{}\" class=\"part hole\" />",
                    p.id,
                    poly_path(&h, true, options.precision)
                ));
            }
        }
    }
    let mut svg = String::new();
    svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" data-doc-id=\"{}\" data-units=\"{}\">\n",
        doc.id, doc.units
    ));
    for i in items {
        svg.push_str("  ");
        svg.push_str(&i);
        svg.push('\n');
    }
    svg.push_str("</svg>\n");
    Ok(svg)
}
