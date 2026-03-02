use craftcad_io::model::*;
use craftcad_io::options::ExportOptions;
use craftcad_io::reasons::{AppError, AppResult};
use craftcad_io::report::IoReport;
use craftcad_io_support::{SupportLevel, SupportMatrix};

fn fmt(v: f64) -> String {
    format!("{:.6}", v)
}
fn header_units(units: Units) -> i32 {
    match units {
        Units::Inch => 1,
        Units::Mm => 4,
    }
}

pub fn export_dxf(
    model: &InternalModel,
    opts: &ExportOptions,
) -> AppResult<(Vec<u8>, Vec<AppError>, IoReport)> {
    let sm = SupportMatrix::load_from_ssot()?;
    let mut warnings = Vec::new();
    let mut report = IoReport::new("dxf");
    let mut out = String::new();
    out.push_str("0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n");
    out.push_str(&format!("{}\n", header_units(opts.target_units)));
    out.push_str("0\nENDSEC\n0\nSECTION\n2\nENTITIES\n");

    for e in &model.entities {
        match e {
            Entity::Path(p) => {
                let all_line = p
                    .segments
                    .iter()
                    .all(|s| matches!(s, Segment2D::Line { .. }));
                if all_line {
                    if sm.level("dxf", "entity_polyline", "export") == SupportLevel::NotSupported {
                        continue;
                    }
                    out.push_str("0\nLWPOLYLINE\n");
                    out.push_str(&format!("8\n{}\n", p.stroke.layer));
                    out.push_str(&format!("90\n{}\n", p.segments.len() + 1));
                    let mut first = true;
                    for s in &p.segments {
                        if let Segment2D::Line { a, b } = s {
                            if first {
                                out.push_str(&format!("10\n{}\n20\n{}\n", fmt(a.x), fmt(a.y)));
                                first = false;
                            }
                            out.push_str(&format!("10\n{}\n20\n{}\n", fmt(b.x), fmt(b.y)));
                        }
                    }
                    if p.closed {
                        out.push_str("70\n1\n");
                    }
                } else {
                    let lvl = sm.level("dxf", "entity_path_unhandled_segment", "export");
                    for r in sm.reasons("dxf", "entity_path_unhandled_segment", "export") {
                        warnings.push(
                            AppError::new(
                                r,
                                if lvl == SupportLevel::NotSupported {
                                    "DXF unhandled segment dropped"
                                } else {
                                    "DXF unhandled segment best-effort; enable_approx recommended"
                                },
                            )
                            .with_context("path_id", p.id.clone()),
                        );
                    }
                }
            }
            Entity::Text(t) => {
                let lvl = sm.level("dxf", "entity_text", "export");
                if lvl == SupportLevel::NotSupported {
                    continue;
                }
                out.push_str("0\nTEXT\n");
                out.push_str(&format!(
                    "8\n{}\n10\n{}\n20\n{}\n40\n{}\n1\n{}\n",
                    t.layer,
                    fmt(t.pos.x),
                    fmt(t.pos.y),
                    fmt(t.size as f64),
                    t.text
                ));
                if lvl == SupportLevel::BestEffort {
                    for r in sm.reasons("dxf", "entity_text", "export") {
                        warnings.push(AppError::new(r, "DXF text exported best-effort"));
                    }
                }
            }
        }
    }
    out.push_str("0\nENDSEC\n0\nEOF\n");
    report.entities_in = model.entities.len();
    report.entities_out = model.entities.len();
    report.determinism_tag = opts.determinism_tag();
    Ok((out.into_bytes(), warnings, report))
}
