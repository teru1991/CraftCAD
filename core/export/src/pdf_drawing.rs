use craftcad_serialize::{Document, Geom2D, Reason, ReasonCode, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingPdfOptions {
    pub title: String,
}
impl Default for DrawingPdfOptions {
    fn default() -> Self {
        Self {
            title: "CraftCAD Drawing".into(),
        }
    }
}

pub fn export_drawing_pdf(doc: &Document, options: &DrawingPdfOptions) -> Result<Vec<u8>> {
    for e in &doc.entities {
        match e.geom {
            Geom2D::Line { .. } | Geom2D::Polyline { .. } => {}
            _ => return Err(Reason::from_code(ReasonCode::ExportUnsupportedEntity)),
        }
    }

    let mut lines = vec![format!("Title: {}", options.title)];
    for p in &doc.parts {
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for v in &p.outline.outer {
            min_x = min_x.min(v.x);
            min_y = min_y.min(v.y);
            max_x = max_x.max(v.x);
            max_y = max_y.max(v.y);
        }
        let w = (max_x - min_x).abs();
        let h = (max_y - min_y).abs();
        lines.push(format!(
            "Part {} ({}) bbox {:.2} x {:.2}",
            p.id, p.name, w, h
        ));
    }

    let text = lines.join("\\n");
    let content = format!(
        "BT /F1 10 Tf 36 760 Td ({}) Tj ET",
        text.replace('(', "\\(")
            .replace(')', "\\)")
            .replace("\n", ") Tj T* (")
    );
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let objs = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [4 0 R] /Count 1 >>".to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents 5 0 R >>".to_string(),
        format!("<< /Length {} >>\nstream\n{}\nendstream", content.len(), content),
        format!("<< /Title ({}) /Producer (CraftCAD) /Subject (units={}) >>", options.title, doc.units),
    ];
    let mut offs = vec![0usize];
    for (i, o) in objs.iter().enumerate() {
        offs.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, o).as_bytes());
    }
    let xref = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for o in offs.iter().skip(1) {
        pdf.extend_from_slice(format!("{:010} 00000 n \n", o).as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer << /Size {} /Root 1 0 R /Info 6 0 R >>\nstartxref\n{}\n%%EOF\n",
            objs.len() + 1,
            xref
        )
        .as_bytes(),
    );
    Ok(pdf)
}
