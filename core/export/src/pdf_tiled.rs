use craftcad_serialize::{Document, Geom2D, Reason, ReasonCode, Result, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PageSize {
    A4,
    Letter,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Orientation {
    Portrait,
    Landscape,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiledPdfOptions {
    pub page_size: PageSize,
    pub orientation: Orientation,
    pub margin_mm: f64,
    pub include_crop_marks: bool,
    pub include_scale_gauge: bool,
    pub title: String,
    pub include_metadata: bool,
}

impl Default for TiledPdfOptions {
    fn default() -> Self {
        Self {
            page_size: PageSize::A4,
            orientation: Orientation::Portrait,
            margin_mm: 10.0,
            include_crop_marks: true,
            include_scale_gauge: true,
            title: "CraftCAD Tiled Export".into(),
            include_metadata: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileLayout {
    pub page_count: usize,
    pub tiles_x: usize,
    pub tiles_y: usize,
    pub page_labels: Vec<String>,
    pub gauge_length_doc_units: f64,
    pub bbox_min: (f64, f64),
    pub bbox_max: (f64, f64),
}

fn mm_to_pt(mm: f64) -> f64 {
    mm / 25.4 * 72.0
}
fn page_mm(size: PageSize, orientation: Orientation) -> (f64, f64) {
    let (w, h) = match size {
        PageSize::A4 => (210.0, 297.0),
        PageSize::Letter => (215.9, 279.4),
    };
    match orientation {
        Orientation::Portrait => (w, h),
        Orientation::Landscape => (h, w),
    }
}

pub fn gauge_length_in_doc_units(units: &str) -> Result<f64> {
    match units {
        "mm" => Ok(100.0),
        "inch" => Ok(100.0 / 25.4),
        _ => Err(Reason::from_code(ReasonCode::ExportUnsupportedFeature)),
    }
}

fn iter_points(doc: &Document) -> impl Iterator<Item = Vec2> + '_ {
    let entities = doc.entities.iter().flat_map(|e| match &e.geom {
        Geom2D::Line { a, b } => vec![a.clone(), b.clone()],
        Geom2D::Polyline { pts, .. } => pts.clone(),
        _ => vec![],
    });
    let part_outer = doc
        .parts
        .iter()
        .flat_map(|p| p.outline.outer.clone().into_iter());
    entities.chain(part_outer)
}

pub fn compute_tiled_layout(doc: &Document, options: &TiledPdfOptions) -> Result<TileLayout> {
    if !options.margin_mm.is_finite() || options.margin_mm < 0.0 {
        return Err(Reason::from_code(ReasonCode::ExportUnsupportedFeature));
    }
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in iter_points(doc) {
        if !p.x.is_finite() || !p.y.is_finite() {
            return Err(Reason::from_code(ReasonCode::ExportUnsupportedEntity));
        }
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    if !min_x.is_finite() {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 1.0;
        max_y = 1.0;
    }

    let (page_w_mm, page_h_mm) = page_mm(options.page_size, options.orientation);
    let draw_w_mm = (page_w_mm - options.margin_mm * 2.0).max(1.0);
    let draw_h_mm = (page_h_mm - options.margin_mm * 2.0).max(1.0);
    let unit_to_mm = if doc.units == "inch" { 25.4 } else { 1.0 };

    let width_mm = ((max_x - min_x).abs() * unit_to_mm).max(1.0);
    let height_mm = ((max_y - min_y).abs() * unit_to_mm).max(1.0);
    let tiles_x = (width_mm / draw_w_mm).ceil() as usize;
    let tiles_y = (height_mm / draw_h_mm).ceil() as usize;
    let mut labels = vec![];
    for y in 0..tiles_y {
        for x in 0..tiles_x {
            labels.push(format!("R{}C{}", y + 1, x + 1));
        }
    }

    Ok(TileLayout {
        page_count: tiles_x * tiles_y,
        tiles_x,
        tiles_y,
        page_labels: labels,
        gauge_length_doc_units: gauge_length_in_doc_units(&doc.units)?,
        bbox_min: (min_x, min_y),
        bbox_max: (max_x, max_y),
    })
}

fn pdf_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

pub fn export_tiled_pdf(doc: &Document, options: &TiledPdfOptions) -> Result<Vec<u8>> {
    for e in &doc.entities {
        match e.geom {
            Geom2D::Line { .. } | Geom2D::Polyline { .. } => {}
            _ => return Err(Reason::from_code(ReasonCode::ExportUnsupportedEntity)),
        }
    }
    let layout = compute_tiled_layout(doc, options)?;
    let (page_w_mm, page_h_mm) = page_mm(options.page_size, options.orientation);
    let page_w_pt = mm_to_pt(page_w_mm);
    let page_h_pt = mm_to_pt(page_h_mm);

    let mut objects = vec![];
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());
    objects.push("<< /Type /Pages /Kids [] /Count 0 >>".to_string()); // placeholder
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string());

    let mut page_refs = vec![];
    let mut page_objs = vec![];
    let mut content_objs = vec![];
    let mut next_id = 4usize;
    for (i, label) in layout.page_labels.iter().enumerate() {
        let content_id = next_id;
        next_id += 1;
        let page_id = next_id;
        next_id += 1;
        page_refs.push(format!("{} 0 R", page_id));

        let mut stream = String::new();
        stream.push_str("BT /F1 10 Tf 20 20 Td ");
        stream.push_str(&format!("({}) Tj ET\n", pdf_escape(label)));
        stream.push_str("BT /F1 9 Tf 20 35 Td ");
        stream.push_str(&format!("(Page {} / {}) Tj ET\n", i + 1, layout.page_count));
        if options.include_scale_gauge {
            let gauge_mm = 100.0;
            let gauge_pt = mm_to_pt(gauge_mm);
            stream.push_str(&format!("50 60 m {} 60 l S\n", 50.0 + gauge_pt));
            stream.push_str("BT /F1 9 Tf 50 70 Td (Gauge 100mm) Tj ET\n");
        }
        if options.include_crop_marks {
            stream.push_str(&format!(
                "5 5 m 20 5 l S\n5 5 m 5 20 l S\n{} 5 m {} 5 l S\n{} 5 m {} 20 l S\n",
                page_w_pt - 5.0,
                page_w_pt - 20.0,
                page_w_pt - 5.0,
                page_w_pt - 5.0
            ));
        }

        content_objs.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            stream.len(),
            stream
        ));
        page_objs.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            page_w_pt, page_h_pt, content_id
        ));
    }

    let pages_obj = format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        page_refs.join(" "),
        layout.page_count
    );
    objects[1] = pages_obj;

    for c in content_objs {
        objects.push(c);
    }
    for p in page_objs {
        objects.push(p);
    }
    let info = if options.include_metadata {
        let job_meta = doc
            .jobs
            .first()
            .map(|j| format!("seed={}", j.seed))
            .unwrap_or_default();
        format!(
            "<< /Title ({}) /Producer (CraftCAD) /Subject (doc_id={} units={} {}) >>",
            pdf_escape(&options.title),
            doc.id,
            doc.units,
            pdf_escape(&job_meta)
        )
    } else {
        "<< /Producer (CraftCAD) >>".to_string()
    };
    objects.push(info);

    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0usize];
    for (i, obj) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, obj).as_bytes());
    }
    let xref_start = pdf.len();
    pdf.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for off in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    let info_id = objects.len();
    pdf.extend_from_slice(
        format!(
            "trailer << /Size {} /Root 1 0 R /Info {} 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            info_id,
            xref_start
        )
        .as_bytes(),
    );
    Ok(pdf)
}
